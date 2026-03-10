use magi::model::popup::{InputContext, InputPopupState, PopupContent, PopupContentCommand};
use magi::model::select_popup::{OnSelect, SelectPopupState};
use magi::msg::update::update;
use magi::msg::{
    FetchCommand, InputMessage, Message, OptionsSource, SelectMessage, ShowSelectPopupConfig,
};

use crate::utils::create_test_model;

mod utils;

// ── FetchRefspecRemotePick routes to ShowFetchRefspecInput ────────────────────

#[test]
fn test_fetch_refspec_remote_pick_routes_to_input() {
    let mut model = create_test_model();

    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Fetch from remote".to_string(),
            vec!["origin".to_string(), "upstream".to_string()],
            OnSelect::FetchRefspecRemotePick,
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowFetchRefspecInput("origin".to_string()))
    );
    assert!(model.popup.is_none());
}

// ── ShowFetchRefspecInput opens an input popup ────────────────────────────────

#[test]
fn test_show_fetch_refspec_input_opens_popup() {
    let mut model = create_test_model();

    let result = update(
        &mut model,
        Message::ShowFetchRefspecInput("origin".to_string()),
    );

    assert_eq!(result, None);
    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref s))
            if matches!(&s.context, InputContext::FetchRefspec { remote } if remote == "origin")
    ));
}

// ── Confirming refspec input dispatches Fetch(FetchRefspecs) ─────────────────

#[test]
fn test_fetch_refspec_input_confirm_dispatches_fetch() {
    let mut model = create_test_model();
    model.popup = Some(PopupContent::Input(InputPopupState {
        input_text: "refs/heads/main:refs/remotes/origin/main".to_string(),
        context: InputContext::FetchRefspec {
            remote: "origin".to_string(),
        },
    }));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Fetch(FetchCommand::FetchRefspecs {
            remote: "origin".to_string(),
            refspecs: "refs/heads/main:refs/remotes/origin/main".to_string(),
        }))
    );
    assert!(model.popup.is_none());
}

// ── SelectPopup::FetchRefspecRemotePick shows a remote select popup ──────────

#[test]
fn test_fetch_refspec_remote_pick_shows_select_popup() {
    let mut model = create_test_model();

    // ShowSelectPopup(FetchRefspecRemotePick) should open a select popup (or error if no remotes)
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Fetch from remote".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::FetchRefspecRemotePick,
        }),
    );

    // No remotes in test repo → error popup
    assert_eq!(result, None);
    assert!(matches!(model.popup, Some(PopupContent::Error { .. })));
}
