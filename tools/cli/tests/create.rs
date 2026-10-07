//! `mineworld create` — a new World Pack a person can validate and run at once (step-08 C7).

mod headless;

use headless::{count_after, fresh, lines, mineworld, stderr, stdout};

#[test]
fn a_created_world_validates_and_runs_with_its_people_doing_things() {
    let root = fresh("create-root");
    let town = root.join("my-town");
    let town = town.to_str().expect("path");

    let created = mineworld(&["create", town]);
    assert!(created.status.success(), "{}", stderr(&created));
    assert!(stdout(&created).contains("World Pack 'my-town'"));

    let validated = mineworld(&["validate", town]);
    assert!(validated.status.success(), "{}", stderr(&validated));
    let report = stdout(&validated);
    assert!(
        report.contains("presence, movement, conversation"),
        "{report}"
    );
    assert!(report.contains("seats      first, second"), "{report}");

    let ran = mineworld(&["run", town, "--headless", "--seed", "1", "--days", "2"]);
    assert!(ran.status.success(), "{}", stderr(&ran));
    let printed = stdout(&ran);
    let activity = lines(&printed, "activity ");
    assert_eq!(activity.len(), 1, "{printed}");
    for seat in ["first", "second"] {
        let moved = count_after(activity[0], &format!("  {seat} move "));
        let talked = count_after(activity[0], &format!("  {seat} move {moved} talk "));
        assert!(moved > 0 && talked > 0, "{seat} acted: {}", activity[0]);
    }
}

#[test]
fn create_never_overwrites_and_refuses_a_name_that_cannot_be_an_id() {
    let existing = fresh("create-existing");
    std::fs::create_dir_all(&existing).expect("a directory");
    std::fs::write(existing.join("keep.txt"), "mine").expect("written");
    let output = mineworld(&["create", existing.to_str().expect("path")]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("already exists"),
        "{}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(existing.join("keep.txt")).expect("still there"),
        "mine"
    );
    assert!(
        !existing.join("world.yaml").exists(),
        "nothing was written into it"
    );

    let root = fresh("create-bad-name");
    let bad = root.join("My Town!");
    let output = mineworld(&["create", bad.to_str().expect("path")]);
    assert!(!output.status.success());
    let complaint = stderr(&output);
    assert!(
        complaint.contains("'My Town!' cannot be a world's id"),
        "{complaint}"
    );
    assert!(!complaint.contains("panicked"), "{complaint}");
    assert!(!bad.exists(), "nothing was created");
}
