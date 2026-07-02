use bevy::prelude::*;

use crate::simulation::{SimulationConfig, SimulationSettings};
use crate::ui::ControlPanelDraft;

use super::applied::AppliedUrlState;
use super::navigation::UrlNavigation;
use super::payload::{decode_applied_state, encode_applied_state};

/// Whether settings were restored from the URL on startup.
#[derive(Resource, Default)]
pub struct UrlHydrated(pub bool);

/// Queue a fragment flush on the next PostUpdate.
#[derive(Resource, Default)]
pub struct PendingUrlSync(pub bool);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UrlHydrateSet;

pub struct UrlSyncPlugin;

impl Plugin for UrlSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlPanelDraft>()
            .init_resource::<PendingUrlSync>()
            .init_resource::<UrlHydrated>()
            .add_systems(
                Startup,
                (hydrate_from_url, queue_initial_url_flush)
                    .chain()
                    .in_set(UrlHydrateSet),
            )
            .add_systems(
                PostUpdate,
                (detect_applied_changes, flush_url_fragment).chain(),
            );
    }
}

fn hydrate_from_url(
    nav: Res<UrlNavigation>,
    mut settings: ResMut<SimulationSettings>,
    mut config: ResMut<SimulationConfig>,
    mut draft: ResMut<ControlPanelDraft>,
    mut hydrated: ResMut<UrlHydrated>,
) {
    let Some(body) = nav.0.current_fragment_body() else {
        return;
    };
    let trimmed = body.trim();
    if !trimmed.starts_with("v=") {
        return;
    }

    let Ok(state) = decode_applied_state(trimmed) else {
        return;
    };

    draft.initial = state.initial.clone();
    state.apply_to_resources(&mut settings, &mut config);
    hydrated.0 = true;
}

fn queue_initial_url_flush(hydrated: Res<UrlHydrated>, mut pending: ResMut<PendingUrlSync>) {
    if !hydrated.0 {
        pending.0 = true;
    }
}

fn detect_applied_changes(
    settings: Res<SimulationSettings>,
    config: Res<SimulationConfig>,
    mut pending: ResMut<PendingUrlSync>,
) {
    if settings.is_changed() || config.is_changed() {
        pending.0 = true;
    }
}

fn flush_url_fragment(
    nav: Res<UrlNavigation>,
    settings: Res<SimulationSettings>,
    config: Res<SimulationConfig>,
    mut pending: ResMut<PendingUrlSync>,
    mut last_token: Local<Option<String>>,
) {
    if !pending.0 {
        return;
    }
    pending.0 = false;

    let state = AppliedUrlState::from_resources(&settings, &config);
    let Ok(query) = encode_applied_state(&state) else {
        return;
    };
    if last_token.as_deref() == Some(query.as_str()) {
        return;
    }
    if nav.0.fragment_equals(query.as_str()) {
        *last_token = Some(query);
        return;
    }
    if nav.0.replace_fragment_body(&query).is_ok() {
        *last_token = Some(query);
    }
}
