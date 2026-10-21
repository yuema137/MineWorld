//! Where the topics and the current notices live: one `notices-board` Process, started by
//! `notices-configured` and never woken (SD-S31b-1).
//!
//! Only the newest notice of each (topic, publisher) is state (SD-S31b-2). Older notices live in the
//! log, in inboxes and in memory, never here, so the state stays bounded: at most 32 × 16 notices, and
//! at most `per_day` issue times per (topic, publisher), pruned to the last 24 hours at each post.

use mineworld_contracts::{PersonId, ProcessTypeId, WorldTime};
use mineworld_kernel::{Process, ProcessKind, WorldRead};
use serde::{Deserialize, Serialize};

use crate::body::NoticeBody;
use crate::event::ConfiguredTopic;
use crate::system::NoticesSystem;
use crate::topic::TopicId;

/// The length of the rolling window `per_day` counts in: 24 hours of world time (Q-S31b-4).
pub const WINDOW_SECONDS: i64 = 86_400;

/// The kind of process the board is, owned — as a type — by [`NoticesSystem`].
pub struct NoticesProcess;

impl ProcessKind for NoticesProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("notices-board");
    type Owner = NoticesSystem;
}

/// A notice's number, allocated from 1 in posting order and never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeId(u64);

impl NoticeId {
    /// The number.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// One notice: what was posted, on what, by whom, when, and until when it is valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    id: NoticeId,
    topic: TopicId,
    publisher: PersonId,
    body: NoticeBody,
    issued_at: WorldTime,
    valid_until: WorldTime,
}

impl Notice {
    pub(crate) const fn new(
        id: NoticeId,
        topic: TopicId,
        publisher: PersonId,
        body: NoticeBody,
        issued_at: WorldTime,
        valid_until: WorldTime,
    ) -> Self {
        Self {
            id,
            topic,
            publisher,
            body,
            issued_at,
            valid_until,
        }
    }

    /// Its number.
    pub const fn id(&self) -> NoticeId {
        self.id
    }

    /// Its topic.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    /// Who posted it.
    pub const fn publisher(&self) -> PersonId {
        self.publisher
    }

    /// What it carries.
    pub const fn body(&self) -> &NoticeBody {
        &self.body
    }

    /// When it was posted.
    pub const fn issued_at(&self) -> WorldTime {
        self.issued_at
    }

    /// The first instant it is no longer valid.
    pub const fn valid_until(&self) -> WorldTime {
        self.valid_until
    }

    /// Valid at `now`: `issued_at ≤ now < valid_until`.
    pub fn valid_at(&self, now: WorldTime) -> bool {
        self.issued_at <= now && now < self.valid_until
    }
}

/// The issue times of one (topic, publisher) within the window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Issued {
    topic: TopicId,
    publisher: PersonId,
    times: Vec<WorldTime>,
}

/// The board's state: what was configured, the next notice number, the newest notice per (topic,
/// publisher), and the recent issue times the rolling limit counts.
///
/// Lists rather than maps keyed by (topic, publisher), because a JSON map key must be a string; each
/// list is kept sorted by (topic, publisher), so the bytes have one form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardState {
    topics: Vec<ConfiguredTopic>,
    next: u64,
    latest: Vec<Notice>,
    issued: Vec<Issued>,
}

impl BoardState {
    pub(crate) const fn configured(topics: Vec<ConfiguredTopic>) -> Self {
        Self {
            topics,
            next: 1,
            latest: Vec::new(),
            issued: Vec::new(),
        }
    }

    /// The topic, if declared.
    pub fn topic(&self, topic: &TopicId) -> Option<&ConfiguredTopic> {
        self.topics
            .iter()
            .find(|declared| declared.topic() == topic)
    }

    /// Every declared topic, in id order.
    pub fn topics(&self) -> &[ConfiguredTopic] {
        &self.topics
    }

    /// The number the next notice gets.
    pub const fn next_id(&self) -> NoticeId {
        NoticeId(self.next)
    }

    /// How many notices `publisher` posted on `topic` in the window ending at `now`: issued after
    /// `now - 24 h`.
    pub fn issued_within(&self, topic: &TopicId, publisher: PersonId, now: WorldTime) -> usize {
        let since = now.seconds() - WINDOW_SECONDS;
        self.issued
            .iter()
            .find(|row| row.topic == *topic && row.publisher == publisher)
            .map_or(0, |row| {
                row.times.iter().filter(|at| at.seconds() > since).count()
            })
    }

    /// The notices current at `now`, in (topic, publisher) order: the newest of each pair, while it is
    /// valid. A newest notice that has expired means the publisher has nothing current on that topic,
    /// because it superseded every older one.
    pub fn current(&self, now: WorldTime) -> impl Iterator<Item = &Notice> {
        self.latest
            .iter()
            .filter(move |notice| notice.valid_at(now))
    }

    /// The board after `notice` is posted: the counter past it, it the newest of its pair, and its
    /// issue time counted, with times older than the window dropped.
    pub(crate) fn posted(mut self, notice: Notice) -> Self {
        let key = (notice.topic.clone(), notice.publisher);
        let at = notice.issued_at;
        self.next = notice.id.0 + 1;
        match self
            .latest
            .binary_search_by(|held| (held.topic.clone(), held.publisher).cmp(&key))
        {
            Ok(index) => self.latest[index] = notice,
            Err(index) => self.latest.insert(index, notice),
        }
        let since = at.seconds() - WINDOW_SECONDS;
        let index = match self
            .issued
            .binary_search_by(|row| (row.topic.clone(), row.publisher).cmp(&key))
        {
            Ok(index) => index,
            Err(index) => {
                self.issued.insert(
                    index,
                    Issued {
                        topic: key.0,
                        publisher: key.1,
                        times: Vec::new(),
                    },
                );
                index
            }
        };
        let times = &mut self.issued[index].times;
        times.retain(|time| time.seconds() > since);
        times.push(at);
        self
    }
}

/// The world's board process, if it has one.
pub(crate) fn board_process<'a>(world: &WorldRead<'a>) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.owner() == <NoticesSystem as mineworld_kernel::SystemIdentity>::ID
            && *process.process_type() == NoticesProcess::PROCESS_TYPE
    })
}

/// The board's state, read back; [`None`] when the world has no configuration.
pub fn board(world: &WorldRead<'_>) -> Option<BoardState> {
    let process = board_process(world)?;
    let bytes = process.state_for::<NoticesProcess>().ok()?;
    serde_json::from_slice(bytes).ok()
}
