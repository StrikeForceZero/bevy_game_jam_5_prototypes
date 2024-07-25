use bevy::color::palettes::basic::TEAL;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashMap;
use itertools::Itertools;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::util::color_ext::ColorExt;
use crate::util::string::{AnyString, AnyUniqueString};

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.init_gizmo_group::<DebugDrawGizmos>()
        .add_systems(Update, draw_lines);
}

#[derive(SystemParam)]
pub struct DebugDrawGizmosSystemParam<'w> {
    config_store: ResMut<'w, GizmoConfigStore>,
}

impl DebugDrawGizmosSystemParam<'_> {
    pub fn get(&mut self) -> &mut DebugDrawGizmos {
        let (_, this) = self.config_store.config_mut::<DebugDrawGizmos>();
        this
    }
}

#[derive(Debug, SmartDefault, Reflect, AutoRegisterType)]
pub struct Point {
    pos: Vec2,
    #[default(Color::Srgba(TEAL))]
    color: Color,
}

#[derive(Debug, Default, Reflect, AutoRegisterType)]
pub struct DebugDrawScope {
    line_strip: Vec<Point>,
}

impl DebugDrawScope {
    pub fn add_point(&mut self, point: Vec2) {
        self.line_strip.push(Point {
            pos: point,
            ..default()
        });
    }
    pub fn add_color_point(&mut self, point: Vec2, color: impl Into<Color>) {
        let color = color.into();
        self.line_strip.push(Point { pos: point, color });
    }
    pub fn clear(&mut self) {
        self.line_strip.clear();
    }
}

#[derive(Debug, Default, Reflect, GizmoConfigGroup, AutoRegisterType)]
pub struct DebugDrawGizmos {
    scopes: HashMap<AnyUniqueString, DebugDrawScope>,
}

impl DebugDrawGizmos {
    pub fn has_scope<'a>(&self, scope: impl Into<&'a AnyUniqueString>) -> bool {
        self.scopes.contains_key(scope.into())
    }
    pub fn remove_scope<'a>(
        &mut self,
        scope: impl Into<&'a AnyUniqueString>,
    ) -> Option<DebugDrawScope> {
        self.scopes.remove(scope.into())
    }
    pub fn get_scope<'a>(&self, scope: impl Into<&'a AnyUniqueString>) -> Option<&DebugDrawScope> {
        self.scopes.get(scope.into())
    }
    pub fn get_scope_mut<'a>(
        &mut self,
        scope: impl Into<&'a AnyUniqueString>,
    ) -> Option<&mut DebugDrawScope> {
        self.scopes.get_mut(scope.into())
    }
    pub fn scope(&mut self, scope: impl Into<AnyUniqueString>) -> &mut DebugDrawScope {
        self.scopes.entry(scope.into()).or_default()
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;

fn draw_lines(mut debug_gizmos: Gizmos<DebugDrawGizmos>) {
    for scope in debug_gizmos.config_ext.scopes.values() {
        for (a, b) in scope.line_strip.iter().tuple_windows() {
            debug_gizmos.line_2d(a.pos, b.pos, [a.color, b.color].avg());
        }
    }
}
