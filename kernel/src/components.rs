//! Component storage: one table per component type, and a write path only its owner can name.
//!
//! The shape is the one `DEP-1` chose after comparing the Rust ECS crates: a `BTreeMap` per
//! component type, keyed by [`EntityId`], behind a narrow API. Not an archetype store, because
//! MineWorld ticks semantically over hundreds of entities and spends its budget on causality and
//! persistence rather than on cache-friendly iteration over tens of thousands; and not
//! `&mut World` handed to every system, because that is exactly how `INV-7` degrades into a
//! review convention.
//!
//! # Writes are gated, reads are open
//!
//! A write takes the owning system's [`WriteToken`] and requires the component type to be
//! [`OwnedBy`] that system, so the gate is the type system's (`KD-1`). A read takes nothing.
//!
//! Reading is open for a reason that matters more than convenience: a component type whose
//! owning system is not installed in this world reads as *absent* rather than as an error. That
//! is what lets a system react to state it does not own without depending on it being there, and
//! it is why uninstalling a system does not break the systems that were reading it.
//!
//! A read hands back `&C`, which is read-only unless `C` itself contains interior mutability —
//! a `Cell`, a `RefCell`, a `Mutex`. A component declared that way can be changed by anybody who
//! can read it, which is a hole in the single-writer rule that the store cannot close: the bound
//! that would rule it out is not available on stable Rust. **A component type must not carry
//! interior mutability.** That is a rule for whoever declares one, and the only one in this
//! module that a reviewer rather than the compiler has to enforce.
//!
//! # What this store does not know
//!
//! It does not know the entity registry. A component is filed under an [`EntityId`], and the
//! store does not ask whether that entity exists, because the pairing of identity with state is
//! the world's business and holding a registry reference here would make every write depend on
//! it. It also never removes state on its own: a destroyed entity's components stay until their
//! owners remove them, because they are their owners' state (`INV-7`) and the kernel deleting
//! them would be the kernel writing another system's component.

#[cfg(test)]
mod tests;

use std::any::Any;
use std::collections::BTreeMap;

use mineworld_contracts::{
    Component, ComponentDeclaration, ComponentRecord, ComponentTypeId, EntityId,
};

use crate::access::{OwnedBy, SystemIdentity, WriteToken};
use crate::error::KernelError;

/// Every component of every declared type, in one world.
///
/// Iteration within a type is in [`EntityId`] order and iteration over the types is in component
/// type name order, so two worlds holding the same state present it identically (`KD-5`).
#[derive(Debug, Default)]
pub struct ComponentStore {
    tables: BTreeMap<ComponentTypeId, Table>,
}

impl ComponentStore {
    /// A world with no component types declared yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares `C`'s table, which is what has to happen before anyone writes a `C`.
    ///
    /// Declaration is separate from the first write on purpose. It happens once, when the owning
    /// system is installed, and it is the single place three things are checked — so that the
    /// write path has nothing left to check:
    ///
    /// 1. **The declaration and the ownership relationship agree.** `C::OWNER` and the token's
    ///    system must be the same system. [`owned_component!`](crate::owned_component) makes
    ///    them the same constant and this cannot fail; a hand-written
    ///    [`OwnedBy`] implementation that claims an owner the component's own declaration does
    ///    not name is refused here, which is the one lie the type system cannot catch.
    /// 2. **No second system claims the component type.** Two writers for one piece of state is
    ///    precisely what `INV-7` forbids, and the contract layer's
    ///    [`ComponentDeclaration::conflicts_with`] is the check for it.
    /// 3. **No second Rust type claims the name.** Two types declaring one
    ///    [`ComponentTypeId`] would each see an empty table and silently lose the other's rows.
    ///
    /// Declaring the same type twice is not an error: a system registering its types again, or a
    /// world re-declaring on load, is not a conflict.
    pub fn declare<C, S>(&mut self, token: &WriteToken<S>) -> Result<(), KernelError>
    where
        C: OwnedBy<S> + 'static,
        S: SystemIdentity,
    {
        let declaration = ComponentDeclaration::of::<C>();
        if *declaration.owner() != token.system() {
            return Err(KernelError::ComponentOwnerDisagreesWithDeclaration {
                component_type: C::COMPONENT_TYPE,
                declared_owner: C::OWNER,
                writing_system: token.system(),
            });
        }

        match self.tables.get(&C::COMPONENT_TYPE) {
            None => {
                self.tables
                    .insert(C::COMPONENT_TYPE, Table::empty_for::<C>(declaration));
                Ok(())
            }
            Some(existing) if existing.declaration.conflicts_with(&declaration) => {
                Err(KernelError::ComponentTypeClaimedByAnotherSystem {
                    component_type: C::COMPONENT_TYPE,
                    declared_by: existing.declaration.owner().clone(),
                    claimed_by: C::OWNER,
                })
            }
            Some(existing)
                if existing.declaration != declaration || existing.rows_of::<C>().is_none() =>
            {
                Err(KernelError::ComponentTypeAlreadyDeclared {
                    component_type: C::COMPONENT_TYPE,
                })
            }
            Some(_) => Ok(()),
        }
    }

    /// Whether this world has a table for that component type — that is, whether the system
    /// owning it is part of this world.
    pub fn is_declared(&self, component_type: &ComponentTypeId) -> bool {
        self.tables.contains_key(component_type)
    }

    /// Every declared component type, in name order: what this world's state is made of.
    pub fn declarations(&self) -> impl Iterator<Item = &ComponentDeclaration> {
        self.tables.values().map(|table| &table.declaration)
    }

    /// Writes `entity`'s `C`, returning what was there before.
    ///
    /// Only `C`'s owner can call this, and the compiler is what says so: `token` is `S`'s and
    /// `C` must be [`OwnedBy`] `S`.
    pub fn insert<C, S>(
        &mut self,
        token: &WriteToken<S>,
        entity: EntityId,
        component: C,
    ) -> Result<Option<C>, KernelError>
    where
        C: OwnedBy<S> + 'static,
        S: SystemIdentity,
    {
        let _ = token;
        Ok(self.rows_mut::<C>()?.insert(entity, component))
    }

    /// `entity`'s `C`, to be changed in place by its owner.
    ///
    /// The mutable read and the write are one operation because they are one right: a system that
    /// may write a component may also change the value it already wrote.
    pub fn get_mut<C, S>(
        &mut self,
        token: &WriteToken<S>,
        entity: EntityId,
    ) -> Result<Option<&mut C>, KernelError>
    where
        C: OwnedBy<S> + 'static,
        S: SystemIdentity,
    {
        let _ = token;
        Ok(self.rows_mut::<C>()?.get_mut(&entity))
    }

    /// Removes `entity`'s `C`, returning it if it was there.
    pub fn remove<C, S>(
        &mut self,
        token: &WriteToken<S>,
        entity: EntityId,
    ) -> Result<Option<C>, KernelError>
    where
        C: OwnedBy<S> + 'static,
        S: SystemIdentity,
    {
        let _ = token;
        Ok(self.rows_mut::<C>()?.remove(&entity))
    }

    /// `entity`'s `C`, if it has one.
    ///
    /// Open to every system. A component type this world has not declared reads as absent, which
    /// is how a system tolerates the absence of one it does not own.
    pub fn get<C: Component + 'static>(&self, entity: EntityId) -> Option<&C> {
        self.rows::<C>()?.get(&entity)
    }

    /// Whether `entity` has a `C`.
    pub fn contains<C: Component + 'static>(&self, entity: EntityId) -> bool {
        self.get::<C>(entity).is_some()
    }

    /// Every `C` in the world, in [`EntityId`] order — never in the order they were written
    /// (`KD-5`).
    ///
    /// Empty for a component type this world has not declared.
    pub fn iter<C: Component + 'static>(&self) -> impl Iterator<Item = (EntityId, &C)> {
        self.rows::<C>()
            .into_iter()
            .flat_map(|rows| rows.iter().map(|(entity, component)| (*entity, component)))
    }

    /// How many entities carry a `C`.
    pub fn count<C: Component + 'static>(&self) -> usize {
        self.rows::<C>().map_or(0, BTreeMap::len)
    }

    fn rows<C: Component + 'static>(&self) -> Option<&BTreeMap<EntityId, C>> {
        self.tables.get(&C::COMPONENT_TYPE)?.rows_of::<C>()
    }

    /// Every component in this world as a record a save can hold: tables in component type name
    /// order, rows in [`EntityId`] order, each payload the component's JSON (`DEP-5` note, S5).
    ///
    /// Only this store can do it: it is the one place that still knows each table's Rust type.
    pub fn records(&self) -> Result<Vec<ComponentRecord>, KernelError> {
        let mut records = Vec::new();
        for table in self.tables.values() {
            records.extend(table.rows.encode()?);
        }
        Ok(records)
    }

    /// A store with the same tables as this one, filled from saved records instead of this store's
    /// rows — or the refusal naming the first record no table here can take.
    ///
    /// Builds new tables and leaves this store untouched, so a refusal changes nothing. Each record
    /// is decoded by the component type its table was declared with, through
    /// [`ComponentRecord::payload_for`], so a record written by a newer or an older schema is refused
    /// with the contract's own `ComponentSchemaTooNew` / `ComponentSchemaOutdated` rather than decoded
    /// on a guess. A record for a component type this world did not install is refused too: it is
    /// state of a system this world is not composed of.
    pub(crate) fn restored(&self, records: Vec<ComponentRecord>) -> Result<Self, KernelError> {
        let mut by_type: BTreeMap<ComponentTypeId, Vec<ComponentRecord>> = BTreeMap::new();
        for record in records {
            if !self.tables.contains_key(record.component_type()) {
                return Err(KernelError::PersistedComponentTypeNotInstalled {
                    component_type: record.component_type().clone(),
                    entity: record.entity(),
                });
            }
            by_type
                .entry(record.component_type().clone())
                .or_default()
                .push(record);
        }

        let mut tables = BTreeMap::new();
        for (component_type, table) in &self.tables {
            let rows = table
                .rows
                .decoded(by_type.remove(component_type).unwrap_or_default())?;
            tables.insert(
                component_type.clone(),
                Table {
                    declaration: table.declaration.clone(),
                    rows,
                },
            );
        }
        Ok(Self { tables })
    }

    /// Whether any table holds a row.
    pub(crate) fn has_rows(&self) -> bool {
        self.tables.values().any(|table| table.rows.len() > 0)
    }

    /// The table `C`'s owner writes through, or the refusal that it was never declared.
    ///
    /// Undeclared is a refusal rather than an implicit declaration, because declaring is where
    /// ownership is checked, and a store that declared types on first write would have to check
    /// on every write instead.
    fn rows_mut<C: Component + 'static>(
        &mut self,
    ) -> Result<&mut BTreeMap<EntityId, C>, KernelError> {
        self.tables
            .get_mut(&C::COMPONENT_TYPE)
            .and_then(Table::rows_of_mut::<C>)
            .ok_or(KernelError::ComponentTypeNotDeclared {
                component_type: C::COMPONENT_TYPE,
            })
    }
}

/// One component type's rows, plus the declaration they were filed under.
///
/// The rows are held as a trait object because a world's component types are known when its
/// systems are installed, not when the kernel is compiled — `DEP-1`'s fourth mismatch with an
/// archetype ECS. Every value inside is still the system's own Rust type: the only thing erased
/// is which type it is, and the only way back in is [`Table::rows_of`], which names it again.
struct Table {
    declaration: ComponentDeclaration,
    rows: Box<dyn ComponentRows>,
}

impl Table {
    fn empty_for<C: Component + 'static>(declaration: ComponentDeclaration) -> Self {
        Self {
            declaration,
            rows: Box::new(BTreeMap::<EntityId, C>::new()),
        }
    }

    fn rows_of<C: Component + 'static>(&self) -> Option<&BTreeMap<EntityId, C>> {
        self.rows.as_any().downcast_ref()
    }

    fn rows_of_mut<C: Component + 'static>(&mut self) -> Option<&mut BTreeMap<EntityId, C>> {
        self.rows.as_any_mut().downcast_mut()
    }
}

/// A table's own `Debug` cannot print its rows, because a component type is not required to be
/// `Debug` — the contract layer asks components to serialize, not to be printable. What a reader
/// of a debug dump needs from the store is which types it holds and how much of each, and that is
/// what this prints.
impl core::fmt::Debug for Table {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} {} owned by {} ({} rows)",
            self.declaration.component_type(),
            self.declaration.schema_version(),
            self.declaration.owner(),
            self.rows.len()
        )
    }
}

/// What the store can do with a table without knowing its component type: count its rows, hand
/// them back to code that names the type, and — because the implementation below *does* know the
/// type — write them down and read them back for a save.
trait ComponentRows {
    fn len(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Every row as a record, in [`EntityId`] order.
    fn encode(&self) -> Result<Vec<ComponentRecord>, KernelError>;
    /// A new, separate table of the same component type holding exactly `records`.
    fn decoded(&self, records: Vec<ComponentRecord>)
    -> Result<Box<dyn ComponentRows>, KernelError>;
}

impl<C: Component + 'static> ComponentRows for BTreeMap<EntityId, C> {
    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn encode(&self) -> Result<Vec<ComponentRecord>, KernelError> {
        self.iter()
            .map(|(entity, component)| {
                let payload = serde_json::to_vec(component).map_err(|error| {
                    KernelError::ComponentNotEncodable {
                        component_type: C::COMPONENT_TYPE,
                        entity: *entity,
                        detail: error.to_string(),
                    }
                })?;
                Ok(ComponentRecord::new::<C>(*entity, payload))
            })
            .collect()
    }

    fn decoded(
        &self,
        records: Vec<ComponentRecord>,
    ) -> Result<Box<dyn ComponentRows>, KernelError> {
        let mut rows: Self = BTreeMap::new();
        for record in records {
            let payload = record.payload_for::<C>()?;
            let component: C = serde_json::from_slice(payload).map_err(|error| {
                KernelError::PersistedComponentUndecodable {
                    component_type: C::COMPONENT_TYPE,
                    entity: record.entity(),
                    detail: error.to_string(),
                }
            })?;
            if rows.insert(record.entity(), component).is_some() {
                return Err(KernelError::PersistedComponentRepeated {
                    component_type: C::COMPONENT_TYPE,
                    entity: record.entity(),
                });
            }
        }
        Ok(Box::new(rows))
    }
}
