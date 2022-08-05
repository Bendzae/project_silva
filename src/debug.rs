use bevy::prelude::*;

#[derive(Component)]
pub struct TestDebugComponent;

#[derive(Bundle)]
pub struct TestBundle {
    pub _t: TestDebugComponent,
    #[bundle]
    pub pbr_bundle: PbrBundle,
}

impl Default for TestBundle {
    fn default() -> TestBundle {
        return TestBundle {
            _t: TestDebugComponent,
            pbr_bundle: PbrBundle::default(),
        };
    }
}
