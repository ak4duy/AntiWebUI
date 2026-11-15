//! The pull-down list, driven like a window would drive it.

use anti_webui_core::Ui;
use anti_webui_core::event::*;
use anti_webui_core::geometry::{PointF, PointI, Scale, SizeI};
use anti_webui_core::platform::{PlatformRequest, PopupParams};
use anti_webui_core::prelude::*;
use anti_webui_widgets::select;
use std::time::{Duration, Instant};

const M: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    logo: false,
};

fn press(ui: &mut Ui, x: f32, y: f32, pressed: bool) {
    ui.handle_event(WindowEvent::PointerButton {
        position: PointF::new(x, y),
        button: PointerButton::Primary,
        pressed,
        modifiers: M,
        time: 0,
    });
    ui.update();
}

fn key(ui: &mut Ui, k: NamedKey) {
    ui.handle_event(WindowEvent::Key(KeyEvent {
        key: Key::Named(k),
        text: None,
        modifiers: M,
        pressed: true,
        repeat: false,
    }));
    ui.update();
}

fn mount(value: Signal<u32>) -> Ui {
    let options = (0..30).map(|i| (i, format!("option {i}"))).collect();
    let mut ui = Ui::new(move || div().p(10.0).w(200.0).child(select(value, options)));
    ui.handle_event(WindowEvent::Resized {
        size: SizeI::new(400, 300),
        scale: Scale::ONE,
    });
    ui.handle_event(WindowEvent::Focused(true));
    ui.update();
    ui
}

/// A list whose names share the letters that are typed.
fn mount_names(value: Signal<u32>) -> Ui {
    let options = vec![
        (0u32, "Apple".to_string()),
        (1, "Adwaita Mono".to_string()),
        (2, "Adwaita Sans".to_string()),
        (3, "Banana".to_string()),
        (4, "avocado".to_string()),
    ];
    let mut ui = Ui::new(move || div().p(10.0_f32).w(200.0).child(select(value, options)));
    ui.handle_event(WindowEvent::Resized {
        size: SizeI::new(400, 300),
        scale: Scale::ONE,
    });
    ui.handle_event(WindowEvent::Focused(true));
    ui.update();
    ui
}

fn typed(ui: &mut Ui, text: &str) {
    for c in text.chars() {
        let s = c.to_string();
        ui.handle_event(WindowEvent::Key(KeyEvent {
            key: Key::Character(s.clone()),
            text: Some(s),
            modifiers: M,
            pressed: true,
            repeat: false,
        }));
    }
    ui.update();
}

fn wait(ui: &mut Ui, milliseconds: u64) {
    ui.tick(Instant::now() + Duration::from_millis(milliseconds));
    ui.update();
}

fn opened(ui: &mut Ui) -> Option<PopupParams> {
    ui.update();
    ui.take_requests().into_iter().find_map(|r| match r {
        PlatformRequest::ShowPopup(p) => Some(p),
        _ => None,
    })
}

fn closed(ui: &mut Ui) -> bool {
    ui.update();
    ui.take_requests()
        .iter()
        .any(|r| matches!(r, PlatformRequest::ClosePopup(_)))
}

/// Plays the platform: below the anchor, at the size it asked for.
fn place(ui: &mut Ui, p: &PopupParams) {
    ui.handle_event(WindowEvent::PopupConfigured {
        id: p.id,
        origin: PointI::new(
            p.anchor.x - p.geometry.x,
            p.anchor.bottom() + p.gap - p.geometry.y,
        ),
        size: p.wanted,
    });
    ui.update();
}

#[test]
fn pressing_opens_it_and_picking_an_option_closes_it() {
    let value = signal(0u32);
    let mut ui = mount(value);
    ui.take_requests();
    press(&mut ui, 50.0, 20.0, true);
    let p = opened(&mut ui).expect("it opens on the press");
    assert_eq!(p.geometry.width, 180, "as wide as what it hangs from");
    assert!(p.geometry.height <= 280, "and scrolls past its height");
    assert!(p.logical.width > p.geometry.width, "room for the shadow");
    press(&mut ui, 50.0, 20.0, false);
    assert!(
        !closed(&mut ui),
        "letting go of the press isn't picking anything"
    );
    place(&mut ui, &p);

    // Option 2, in the window's coordinates.
    let y = p.anchor.bottom() as f32 + 4.0 + 5.0 + 2.5 * 30.0;
    press(&mut ui, 50.0, y, true);
    press(&mut ui, 50.0, y, false);
    assert_eq!(value.get(), 2);
    assert!(closed(&mut ui));
}

#[test]
fn pressing_it_again_closes_it_instead_of_opening_it_again() {
    let value = signal(0u32);
    let mut ui = mount(value);
    press(&mut ui, 50.0, 20.0, true);
    press(&mut ui, 50.0, 20.0, false);
    let p = opened(&mut ui).unwrap();
    place(&mut ui, &p);
    press(&mut ui, 50.0, 20.0, true);
    assert!(closed(&mut ui));
    press(&mut ui, 50.0, 20.0, false);
    assert!(opened(&mut ui).is_none());
    assert_eq!(value.get(), 0);
}

#[test]
fn the_keyboard_works_from_where_the_focus_stays() {
    let value = signal(3u32);
    let mut ui = mount(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    key(&mut ui, NamedKey::Enter);
    let p = opened(&mut ui).expect("enter opens it");
    place(&mut ui, &p);
    key(&mut ui, NamedKey::ArrowDown);
    key(&mut ui, NamedKey::ArrowDown);
    assert_eq!(value.get(), 3, "nothing is picked yet");
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 5);
    assert!(closed(&mut ui));

    key(&mut ui, NamedKey::ArrowDown);
    let p = opened(&mut ui).expect("arrow down opens it");
    place(&mut ui, &p);
    key(&mut ui, NamedKey::End);
    key(&mut ui, NamedKey::Escape);
    assert!(closed(&mut ui));
    assert_eq!(value.get(), 5, "escape leaves it as it was");
}

#[test]
fn typing_a_letter_opens_it_on_the_next_name_with_that_letter() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "a");
    let p = opened(&mut ui).expect("typing opens it");
    assert_eq!(value.get(), 0, "nothing is picked yet");
    place(&mut ui, &p);
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 1, "the first A after the one chosen");
    assert!(closed(&mut ui));
}

#[test]
fn typing_the_same_letter_again_cycles_and_wraps() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "a");
    let p = opened(&mut ui).unwrap();
    place(&mut ui, &p);
    typed(&mut ui, "aa");
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 4, "past the last A, around to avocado");

    key(&mut ui, NamedKey::ArrowDown);
    let p = opened(&mut ui).unwrap();
    place(&mut ui, &p);
    typed(&mut ui, "a");
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 0, "and around to the first");
}

#[test]
fn a_whole_prefix_lands_on_the_name_it_starts() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "adwaita");
    key(&mut ui, NamedKey::Space);
    typed(&mut ui, "s");
    let p = opened(&mut ui).expect("it opened on the first letter");
    place(&mut ui, &p);
    assert_eq!(value.get(), 0, "space went into the name, not into choosing");
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 2);
}

#[test]
fn space_with_nothing_typed_still_opens_it() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    key(&mut ui, NamedKey::Space);
    let p = opened(&mut ui).expect("space opens it");
    place(&mut ui, &p);
    key(&mut ui, NamedKey::Space);
    assert_eq!(value.get(), 0);
    assert!(closed(&mut ui));
}

#[test]
fn the_typed_run_forgets_itself_after_a_second() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "a");
    let p = opened(&mut ui).unwrap();
    place(&mut ui, &p);
    wait(&mut ui, 1100);
    typed(&mut ui, "a");
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 2, "the run started over");
}

#[test]
fn ctrl_a_is_not_typing() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    ui.handle_event(WindowEvent::Key(KeyEvent {
        key: Key::Character("a".to_string()),
        text: None,
        modifiers: Modifiers { ctrl: true, ..M },
        pressed: true,
        repeat: false,
    }));
    ui.update();
    assert!(opened(&mut ui).is_none(), "ctrl+a leaves it closed");
    assert_eq!(value.get(), 0);
}

#[test]
fn a_letter_that_matches_nothing_leaves_the_lit_one_alone() {
    let value = signal(0u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "b");
    let p = opened(&mut ui).unwrap();
    place(&mut ui, &p);
    typed(&mut ui, "z");
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 3, "still on Banana");
}

#[test]
fn a_letter_with_one_name_stays_on_it() {
    let value = signal(3u32);
    let mut ui = mount_names(value);
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "b");
    let p = opened(&mut ui).unwrap();
    place(&mut ui, &p);
    key(&mut ui, NamedKey::Enter);
    assert_eq!(value.get(), 3);
}

#[test]
fn a_typed_run_that_outlives_the_list_does_not_panic() {
    let value = signal(0u32);
    let visible = signal(true);
    let options = vec![
        (0u32, "Apple".to_string()),
        (1, "Adwaita Mono".to_string()),
    ];
    let mut ui = Ui::new(move || {
        div().p(10.0_f32).w(200.0).child(show(
            move || visible.get(),
            move || div().child(select(value, options.clone())),
        ))
    });
    ui.handle_event(WindowEvent::Resized {
        size: SizeI::new(400, 300),
        scale: Scale::ONE,
    });
    ui.handle_event(WindowEvent::Focused(true));
    ui.update();
    key(&mut ui, NamedKey::Tab);
    ui.take_requests();
    typed(&mut ui, "a");
    visible.set(false);
    ui.update();
    wait(&mut ui, 1100);
    assert_eq!(value.get(), 0);
}
