import StackTrace from "stacktrace-js";

function displayError(prefix, error) {
    const el = document.getElementById("errors");
    el.className = "errorsHappened";

    StackTrace.fromError(error).then(stackFrames => {
        document.getElementById("errorsloading").className = "loaded";
        el.insertAdjacentHTML("beforeend", `<h2>${prefix}: ${error.message}</h2>`);
        for (let frame of stackFrames) {
            const fileName = frame.fileName.replace(window.location.origin, "");
            el.insertAdjacentHTML("beforeend", `${frame.functionName} in ${fileName} ${frame.lineNumber}:${frame.columnNumber}<br/>`)
        }
        el.insertAdjacentHTML("beforeend", '<br/>');
    }).catch(() => {
        document.getElementById("errorsloading").className = "loaded";
        el.insertAdjacentHTML("beforeend", `<h2>${prefix}: ${error.message}</h2>`);
        el.insertAdjacentHTML("beforeend", 'failed to gather error origin :(');
    });
}

window.onerror = function (msg, file, line, col, error) {
    displayError("Error", error || { message: msg });
};

window.addEventListener('unhandledrejection', function (e) {
    displayError("Unhandled Rejection", e.reason);
});

import Monet from 'monet';

if (process.env.NODE_ENV === 'production' && window.__REACT_DEVTOOLS_GLOBAL_HOOK__) {
    window.__REACT_DEVTOOLS_GLOBAL_HOOK__.inject = function () { };
    window.__REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE = function () { };
}

import * as React from 'react';
import * as ReactDOM from 'react-dom';

import ContainerDimensions from 'react-container-dimensions';
import update from 'immutability-helper';
import * as Camera from './camera/Camera';
import * as Planning from './planning_browser/Planning';
import * as Transport from './transport_browser/Transport';
import * as LandUse from './land_use_browser/LandUse';
import * as Households from './households_browser/Households';
import * as Vegetation from './vegetation_browser/Vegetation';
import MainUIModes from './uiModes';
import * as Time from './time_browser/Time';
import * as Debug from './debug/Debug';
import * as Settings from './settings';
import MainMenu, * as Menu from './menu';
import * as Utils from './browser_utils/Utils';
import Stage from './stage/Stage';
import colors from './colors';

type Gesture = { intent: Intent };
type Intent = { Road?: { path: EditArcLinePath }, Zone?: { boundary: EditArcLinePath } };
type EditArcLinePath = { corners: { position: [number, number] }[] };

interface CBRustAPI {
    start(): void;

    set_intent(projectId: string, gestureId: string, intent: Intent, doneAdding: boolean): void;

    move_gesture_point(projectId: string, gestureId: string, pointIdx: number, position: [number, number], doneMoving: boolean): void;
    start_new_gesture(projectId: string, gestureId: string, intent: Intent): void;
    with_control_point_added(intent: Intent, point: [number, number], addToEnd: boolean): Intent;
    insert_control_point(projectId: string, gestureId: string, point: [number, number], doneInserting: boolean): void;
    split_gesture(projectId: string, gestureId: string, point: [number, number], doneSplitting: boolean): void;
    set_n_lanes(projectId: string, gestureId: string, nLanesForward: number, nLanesBackward: number, doneChanging: boolean): void;

    get_newest_log_messages(): void;
    plan_grid(projectId: string, n: number, nLanes: number, spacing: number): void;
    spawn_cars(triesPerLane: number): void;
    get_building_info(buildingId: string): void;
    get_household_info(householdId: string): void;
    set_sim_speed(newSpeed: number): void;
    point_in_area(point: [number, number], area: unknown): boolean;
    point_close_to_path(
        point: [number, number],
        path: unknown,
        maxDistanceRight: number,
        maxDistanceLeft: number,
    ): [[number, number], [number, number], [number, number]] | undefined;
}

interface WasmBindgenGlobal extends CBRustAPI {
    (input?: string): Promise<unknown>;
}

declare global {
    interface Window {
        update: typeof update;
        cbRustBrowser: CBRustAPI;
        cbColors: typeof colors;
        cbversion: string;
        __REACT_DEVTOOLS_GLOBAL_HOOK__?: any;
    }

    const process: { env: any }
}

export type SharedState = {
    planning: Planning.PlanningSharedState,
    transport: any,
    landUse: any,
    households: any,
    vegetation: any,
    debug: any,
    system: {
        networkingTurns: string
    },
    rendering: {
        enabled: boolean
    },
    time: any,
    camera: any,

    settings: any
}

export type SetSharedState = (updater: (oldState: SharedState) => SharedState) => void;

export function ToToolPortal(props: { children: React.ReactNode }) {
    const toolsRoot = document.getElementById('tools-root');
    return ReactDOM.createPortal(props.children, toolsRoot);
}

export function ToWindowPortal(props: { children: React.ReactNode }) {
    const windowsRoot = document.getElementById('windows-root');
    return ReactDOM.createPortal(props.children, windowsRoot);
}

window.update = update;
window.cbColors = colors;

function loadRustBrowser(): Promise<CBRustAPI> {
    const importRuntimeModule = Function("path", "return import(path);") as (path: string) => Promise<{
        default: () => Promise<unknown>;
    } & CBRustAPI>;
    return importRuntimeModule('/cb_browser_ui.js').then(wasmModule =>
        wasmModule.default().then(() => wasmModule)
    );
}

loadRustBrowser().then(cbRustBrowser => {
    window.cbRustBrowser = cbRustBrowser;

    const settingSpecs = {
        camera: Camera.settingSpec,
        debug: Debug.settingsSpec,
        planning: Planning.settingsSpec,
        rendering: {
            retinaFactor: { default: 2, description: "Oversampling/Retina Factor", min: 0.5, max: 4.0, step: 0.1 }
        }
    };

    class CityboundReactApp extends React.Component {
        renderer: React.RefObject<Monet>;
        state: SharedState;
        boundSetState: SetSharedState;

        constructor(props) {
            super(props);

            this.state = {
                planning: Planning.initialState,
                transport: Transport.initialState,
                landUse: LandUse.initialState,
                households: Households.initialState,
                vegetation: Vegetation.initialState,
                debug: Debug.initialState,
                system: {
                    networkingTurns: ""
                },
                rendering: {
                    enabled: true
                },
                time: Time.initialState,
                camera: Camera.initialState,

                settings: Settings.loadSettings(settingSpecs)
            }

            this.renderer = React.createRef();
            this.boundSetState = this.setState.bind(this);
        }

        componentDidMount() {
            Camera.bindInputs(this.state, this.boundSetState);
            Debug.bindInputs(this.state, this.boundSetState);
        }

        onFrame() {
            if (this.state.rendering.enabled) {
                Camera.onFrame(this.state, this.boundSetState);

                const renderer = this.renderer.current as any;
                if (!renderer) {
                    return;
                }

                if (renderer.state && renderer.state.glError) {
                    this.boundSetState(oldState => update(oldState, {
                        rendering: { enabled: { "$set": false } }
                    }));
                    return;
                }

                try {
                    renderer.renderFrame();
                } catch (error) {
                    console.error(error);
                    this.boundSetState(oldState => update(oldState, {
                        rendering: { enabled: { "$set": false } }
                    }));
                }
            }
        }

        render() {
            let layers = [];
            let interactive3Dshapes = [];

            return <div style={{ width: "100%", height: "100%" }}>
                <ContainerDimensions style={{ width: "100%", height: "100%", position: "relative" }}>{({ width, height }) =>
                    <Camera.Camera state={this.state} {... { width, height }}>
                        {({ project2dTo3d, project3dTo2d, view, perspective }) =>
                            <div style={{ width, height }}>
                                <Utils.SettingsContext.Provider value={this.state.settings} >
                                    <ToWindowPortal>
                                        <Time.Windows state={this.state} setState={this.boundSetState} />
                                        <Debug.Windows state={this.state} setState={this.boundSetState} />
                                    </ToWindowPortal>

                                    <Utils.Interactive3DContext.Provider value={interactive3Dshapes} >
                                        <Utils.RenderContext.Provider value={layers} >
                                            <MainUIModes state={this.state} setState={this.boundSetState} project2dTo3d={project2dTo3d} project3dTo2d={project3dTo2d} />

                                            <LandUse.Layers state={this.state} />
                                            <Vegetation.Layers state={this.state} />
                                            <Transport.Layers state={this.state} />

                                        </Utils.RenderContext.Provider>
                                    </Utils.Interactive3DContext.Provider>

                                    <MainMenu state={this.state} setState={this.boundSetState} settingSpecs={settingSpecs} />
                                </Utils.SettingsContext.Provider>

                                <Monet key="canvas" ref={this.renderer}
                                    retinaFactor={this.state.settings.rendering.retinaFactor}
                                    clearColor={[...colors.grass, 1.0]}
                                    {... { layers, width, height, viewMatrix: view, perspectiveMatrix: perspective }} />

                                <Stage key="stage"
                                    requestedProjections={this.state.requestedProjections}
                                    style={{ width, height, position: "absolute", top: 0, left: 0 }}
                                    onWheel={e => {
                                        Camera.onWheel(e, this.state, this.boundSetState);
                                        e.preventDefault();
                                        return false;
                                    }}
                                    onMouseMove={e => {
                                        Camera.onMouseMove(e, this.state, this.boundSetState);
                                    }}
                                    {...{ interactables: interactive3Dshapes, width, height, project2dTo3d }}
                                />
                            </div>
                        }</Camera.Camera>
                }</ContainerDimensions>
            </div >;
        }
    }

    window.cbReactApp = ReactDOM.render(<CityboundReactApp />, document.getElementById('layers-root'));

    cbRustBrowser.start();
});
