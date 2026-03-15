#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use serde::de::DeserializeOwned;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use serde::Serialize;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use stdweb::serde::Serde;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn ensure_runtime() {
    stdweb::initialize();
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn to_js<T: Serialize>(value: T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&value).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = start)]
pub fn start_export() {
    ensure_runtime();
    crate::start();
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = point_in_area)]
pub fn point_in_area_export(point: JsValue, area: JsValue) -> Result<bool, JsValue> {
    ensure_runtime();
    Ok(crate::point_in_area(Serde(from_js(point)?), Serde(from_js(area)?)))
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = point_close_to_path)]
pub fn point_close_to_path_export(
    point: JsValue,
    path: JsValue,
    max_distance_right: JsValue,
    max_distance_left: JsValue,
) -> Result<JsValue, JsValue> {
    ensure_runtime();
    let result = crate::point_close_to_path(
        Serde(from_js(point)?),
        Serde(from_js(path)?),
        Serde(from_js(max_distance_right)?),
        Serde(from_js(max_distance_left)?),
    );
    to_js(result.0)
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = plan_grid)]
pub fn plan_grid_export(
    project_id: JsValue,
    n: i32,
    n_lanes: u32,
    spacing: f64,
) -> Result<(), JsValue> {
    ensure_runtime();
    crate::debug::plan_grid(
        Serde(from_js(project_id)?),
        Serde(n as isize),
        Serde(n_lanes as u8),
        Serde(spacing as f32),
    );
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = spawn_cars)]
pub fn spawn_cars_export(tries_per_lane: u32) {
    ensure_runtime();
    crate::debug::spawn_cars(tries_per_lane as usize);
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = get_newest_log_messages)]
pub fn get_newest_log_messages_export() {
    ensure_runtime();
    crate::debug::get_newest_log_messages();
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = get_household_info)]
pub fn get_household_info_export(household_id: JsValue) -> Result<(), JsValue> {
    ensure_runtime();
    crate::households_browser::get_household_info(Serde(from_js(household_id)?));
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = get_building_info)]
pub fn get_building_info_export(building_id: JsValue) -> Result<(), JsValue> {
    ensure_runtime();
    crate::land_use_browser::get_building_info(Serde(from_js(building_id)?));
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = set_sim_speed)]
pub fn set_sim_speed_export(new_speed: u16) {
    ensure_runtime();
    crate::time_browser::set_sim_speed(new_speed);
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = start_new_gesture)]
pub fn start_new_gesture_export(
    project_id: JsValue,
    gesture_id: JsValue,
    intent: JsValue,
) -> Result<(), JsValue> {
    ensure_runtime();
    crate::planning_browser::start_new_gesture(
        Serde(from_js(project_id)?),
        Serde(from_js(gesture_id)?),
        Serde(from_js(intent)?),
    );
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = with_control_point_added)]
pub fn with_control_point_added_export(
    intent: JsValue,
    point: JsValue,
    add_to_end: bool,
) -> Result<JsValue, JsValue> {
    ensure_runtime();
    let result = crate::planning_browser::with_control_point_added(
        Serde(from_js(intent)?),
        Serde(from_js(point)?),
        add_to_end,
    );
    to_js(result.0)
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = set_intent)]
pub fn set_intent_export(
    project_id: JsValue,
    gesture_id: JsValue,
    intent: JsValue,
    commit: bool,
) -> Result<(), JsValue> {
    ensure_runtime();
    crate::planning_browser::set_intent(
        Serde(from_js(project_id)?),
        Serde(from_js(gesture_id)?),
        Serde(from_js(intent)?),
        commit,
    );
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = undo)]
pub fn undo_export(project_id: JsValue) -> Result<(), JsValue> {
    ensure_runtime();
    crate::planning_browser::undo(Serde(from_js(project_id)?));
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = redo)]
pub fn redo_export(project_id: JsValue) -> Result<(), JsValue> {
    ensure_runtime();
    crate::planning_browser::redo(Serde(from_js(project_id)?));
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = implement_project)]
pub fn implement_project_export(project_id: JsValue) -> Result<(), JsValue> {
    ensure_runtime();
    crate::planning_browser::implement_project(Serde(from_js(project_id)?));
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = start_new_project)]
pub fn start_new_project_export(project_id: JsValue) -> Result<(), JsValue> {
    ensure_runtime();
    crate::planning_browser::start_new_project(Serde(from_js(project_id)?));
    Ok(())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = new_road_intent)]
pub fn new_road_intent_export(
    n_lanes_forward: u32,
    n_lanes_backward: u32,
) -> Result<JsValue, JsValue> {
    ensure_runtime();
    let result = crate::planning_browser::new_road_intent(
        n_lanes_forward as usize,
        n_lanes_backward as usize,
    );
    to_js(result.0)
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen(js_name = new_zone_intent)]
pub fn new_zone_intent_export(new_land_use: JsValue) -> Result<JsValue, JsValue> {
    ensure_runtime();
    let result = crate::planning_browser::new_zone_intent(Serde(from_js(new_land_use)?));
    to_js(result.0)
}
