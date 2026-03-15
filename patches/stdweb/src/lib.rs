//! The goal of this crate is to provide Rust bindings to the Web APIs and to allow
//! a high degree of interoperability between Rust and JavaScript.
//!
//! ## Examples
//!
//! You can directly embed JavaScript code into Rust:
//!
//! ```rust
//! let message = "Hello, 世界!";
//! let result = js! {
//!     alert( @{message} );
//!     return 2 + 2 * 2;
//! };
//!
//! println!( "2 + 2 * 2 = {:?}", result );
//! ```
//!
//! Closures are also supported:
//!
//! ```rust
//! let print_hello = |name: String| {
//!     println!( "Hello, {}!", name );
//! };
//!
//! js! {
//!     var print_hello = @{print_hello};
//!     print_hello( "Bob" );
//!     print_hello.drop(); // Necessary to clean up the closure on Rust's side.
//! }
//! ```
//!
//! You can also pass arbitrary structures thanks to [serde]:
//!
//! ```rust
//! #[derive(Serialize)]
//! struct Person {
//!     name: String,
//!     age: i32
//! }
//!
//! js_serializable!( Person );
//!
//! js! {
//!     var person = @{person};
//!     console.log( person.name + " is " + person.age + " years old." );
//! };
//! ```
//!
//! [serde]: https://serde.rs/
//!
//! This crate also exposes a number of Web APIs, for example:
//!
//! ```rust
//! let button = document().query_selector( "#hide-button" ).unwrap().unwrap();
//! button.add_event_listener( move |_: ClickEvent| {
//!     for anchor in document().query_selector_all( "#main a" ) {
//!         js!( @{anchor}.style = "display: none;"; );
//!     }
//! });
//! ```
//!
//! Exposing Rust functions to JavaScript is supported too:
//!
//! ```rust
//! #[js_export]
//! fn hash( string: String ) -> String {
//!     let mut hasher = Sha1::new();
//!     hasher.update( string.as_bytes() );
//!     hasher.digest().to_string()
//! }
//! ```
//!
//! Then you can do this from Node.js:
//!
//! ```js
//! var hasher = require( "hasher.js" ); // Where `hasher.js` is generated from Rust code.
//! console.log( hasher.hash( "Hello world!" ) );
//! ```
//!
//! Or you can take the same `.js` file and use it in a web browser:
//!
//! ```html
//! <script src="hasher.js"></script>
//! <script>
//!     Rust.hasher.then( function( hasher ) {
//!         console.log( hasher.hash( "Hello world!" ) );
//!     });
//! </script>
//! ```
//!
//! If you're using [Parcel] you can also use our [experimental Parcel plugin];
//! first do this in your existing Parcel project:
//!
//!     $ npm install --save parcel-plugin-cargo-web
//!
//! And then simply:
//!
//! ```js
//! import hasher from "./hasher/Cargo.toml";
//! console.log( hasher.hash( "Hello world!" ) );
//! ```
//!
//! [Parcel]: https://parceljs.org/
//! [experimental Parcel plugin]: https://github.com/koute/parcel-plugin-cargo-web

#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_numeric_casts,
    unused_import_braces
)]
#![allow(semicolon_in_expressions_from_macros)]
#![cfg_attr(
    all(test, rust_nightly),
    feature(linkage) // Needed for async tests.
)]
#![cfg_attr(feature = "nightly", feature(never_type))]
#![recursion_limit="1500"]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde as serde_crate;

#[cfg(any(test, feature = "serde_json"))]
extern crate serde_json;

#[cfg(all(test, feature = "serde"))]
#[macro_use]
extern crate serde_derive;

#[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web)))]
extern crate wasm_bindgen;

#[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web), test))]
#[macro_use]
extern crate wasm_bindgen_test;

#[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web), test))]
#[macro_use]
extern crate stdweb_internal_test_macro;

extern crate stdweb_internal_macros;

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown"
))]
pub use stdweb_internal_macros::js_export;

pub use stdweb_internal_macros::async_test;

#[cfg(feature = "futures-support")]
extern crate futures_core;

#[cfg(feature = "futures-support")]
extern crate futures_util;

#[cfg(feature = "futures-support")]
extern crate futures_channel;

#[cfg(feature = "futures-support")]
extern crate futures_executor;

#[macro_use]
extern crate stdweb_derive;
#[macro_use]
extern crate stdweb_internal_runtime;

extern crate discard;

#[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web), test))]
wasm_bindgen_test_configure!( run_in_browser );

#[macro_use]
mod webcore;
mod webapi;
mod ecosystem;

// This is here so that our procedural macros
// can work within the crate.
pub(crate) mod stdweb {
    pub use super::*;
}

pub use crate::webcore::initialization::{
    initialize,
    event_loop
};
pub use crate::webcore::value::{
    Undefined,
    Null,
    Value,
    Reference
};
pub use crate::webcore::number::Number;
pub use crate::webcore::object::Object;
pub use crate::webcore::array::Array;
pub use crate::webcore::symbol::Symbol;

pub use crate::webcore::unsafe_typed_array::UnsafeTypedArray;
pub use crate::webcore::mutfn::Mut;
pub use crate::webcore::once::Once;
pub use crate::webcore::instance_of::InstanceOf;
pub use crate::webcore::reference_type::ReferenceType;
pub use crate::webcore::serialization::JsSerialize;

pub use crate::webcore::discard::DiscardOnDrop;

#[cfg(feature = "experimental_features_which_may_break_on_minor_version_bumps")]
pub use crate::webcore::promise::{TypedPromise, Promise, DoneHandle};

#[cfg(all(
    feature = "futures-support",
    feature = "experimental_features_which_may_break_on_minor_version_bumps"
))]
pub use crate::webcore::promise_future::{PromiseFuture, spawn_local, print_error_panic, unwrap_future};

#[cfg(feature = "serde")]
/// A module with serde-related APIs.
pub mod serde {
    pub use crate::ecosystem::serde::{
        ConversionError,
        Serde
    };
}

/// A module with bindings to the Web APIs.
pub mod web {
    #[cfg(feature = "futures-support")]
    pub use webapi::timer_future::{
        Wait,
        wait,
        IntervalBuffered,
        interval_buffered
    };

    pub use crate::webapi::window::{
        Window,
        window
    };
    pub use crate::webapi::document::{
        Document,
        document
    };
    pub use crate::webapi::global::{
        set_timeout,
        alert,
        confirm
    };
    pub use crate::webapi::cross_origin_setting::CrossOriginSetting;
    pub use crate::webapi::date::Date;
    pub use crate::webapi::event_target::{IEventTarget, EventTarget, EventListenerHandle};
    pub use crate::webapi::window::RequestAnimationFrameHandle;
    pub use crate::webapi::node::{INode, Node, CloneKind, NodeType};
    pub use crate::webapi::element::{IElement, Element};
    pub use crate::webapi::document_fragment::DocumentFragment;
    pub use crate::webapi::text_node::TextNode;
    pub use crate::webapi::html_element::{IHtmlElement, HtmlElement, Rect};
    pub use crate::webapi::window_or_worker::IWindowOrWorker;
    pub use crate::webapi::parent_node::IParentNode;
    pub use crate::webapi::slotable::ISlotable;
    pub use crate::webapi::non_element_parent_node::INonElementParentNode;
    pub use crate::webapi::token_list::TokenList;
    pub use crate::webapi::node_list::NodeList;
    pub use crate::webapi::string_map::StringMap;
    pub use crate::webapi::storage::Storage;
    pub use crate::webapi::location::Location;
    pub use crate::webapi::array_buffer::ArrayBuffer;
    pub use crate::webapi::typed_array::TypedArray;
    pub use crate::webapi::file::File;
    pub use crate::webapi::file_reader::{FileReader, FileReaderResult, FileReaderReadyState};
    pub use crate::webapi::file_list::FileList;
    pub use crate::webapi::history::History;
    pub use crate::webapi::web_socket::{WebSocket, SocketCloseCode, SocketBinaryType, SocketReadyState};
    pub use crate::webapi::rendering_context::{RenderingContext, CanvasRenderingContext2d, CanvasGradient, CanvasPattern, CanvasStyle, CompositeOperation, FillRule, ImageData, LineCap, LineJoin, Repetition, TextAlign, TextBaseline, TextMetrics};
    pub use crate::webapi::mutation_observer::{MutationObserver, MutationObserverHandle, MutationObserverInit, MutationRecord};
    pub use crate::webapi::xml_http_request::{XmlHttpRequest, XhrReadyState, XhrResponseType};
    pub use crate::webapi::blob::{IBlob, Blob};
    pub use crate::webapi::html_collection::HtmlCollection;
    pub use crate::webapi::child_node::IChildNode;
    pub use crate::webapi::gamepad::{Gamepad, GamepadButton, GamepadMappingType};
    pub use crate::webapi::touch::{Touch, TouchType};
    pub use crate::webapi::selection::Selection;
    pub use crate::webapi::shadow_root::{ShadowRootMode, ShadowRoot};
    pub use crate::webapi::html_elements::SlotContentKind;
    pub use crate::webapi::form_data::{FormData, FormDataEntry};
    pub use crate::webapi::window_or_worker::TimeoutHandle;

    /// A module containing error types.
    pub mod error {
        pub use crate::webapi::dom_exception::{
            IDomException,
            DomException,
            HierarchyRequestError,
            IndexSizeError,
            InvalidAccessError,
            InvalidStateError,
            NotFoundError,
            NotSupportedError,
            SecurityError,
            SyntaxError,
            InvalidCharacterError,
            AbortError
        };

        pub use crate::webapi::error::{
            IError,
            Error,
            TypeError
        };

        pub use crate::webapi::rendering_context::{AddColorStopError, DrawImageError, GetImageDataError};
        pub use crate::webapi::html_elements::UnknownValueError;
        pub use crate::webapi::xml_http_request::XhrSetResponseTypeError;
    }

    /// A module containing HTML DOM elements.
    pub mod html_element {
        pub use crate::webapi::html_elements::ImageElement;
        pub use crate::webapi::html_elements::InputElement;
        pub use crate::webapi::html_elements::TextAreaElement;
        pub use crate::webapi::html_elements::CanvasElement;
        pub use crate::webapi::html_elements::SelectElement;
        pub use crate::webapi::html_elements::OptionElement;
        pub use crate::webapi::html_elements::TemplateElement;
        pub use crate::webapi::html_elements::SlotElement;
    }

    /// A module containing JavaScript DOM events.
    pub mod event {
        pub use crate::webapi::event::{
            IEvent,
            IUiEvent,
            ConcreteEvent,

            UnloadEvent,
            BeforeUnloadEvent,

            FullscreenChangeEvent,

            EventPhase
        };

        pub use crate::webapi::events::mouse::{
            IMouseEvent,
            ClickEvent,
            AuxClickEvent,
            ContextMenuEvent,
            DoubleClickEvent,
            MouseDownEvent,
            MouseUpEvent,
            MouseMoveEvent,
            MouseOverEvent,
            MouseOutEvent,
            MouseEnterEvent,
            MouseLeaveEvent,
            MouseWheelEvent,
            MouseWheelDeltaMode,
            MouseButton,
            MouseButtonsState
        };

        pub use crate::webapi::events::touch::{
            ITouchEvent,
            TouchEvent,
            TouchMove,
            TouchLeave,
            TouchEnter,
            TouchEnd,
            TouchCancel,
            TouchStart,
        };

        pub use crate::webapi::events::pointer::{
            IPointerEvent,
            PointerOverEvent,
            PointerEnterEvent,
            PointerDownEvent,
            PointerMoveEvent,
            PointerUpEvent,
            PointerCancelEvent,
            PointerOutEvent,
            PointerLeaveEvent,
            GotPointerCaptureEvent,
            LostPointerCaptureEvent,
            PointerLockChangeEvent,
            PointerLockErrorEvent
        };

        pub use crate::webapi::events::keyboard::{
            IKeyboardEvent,
            KeyPressEvent,
            KeyDownEvent,
            KeyUpEvent,

            KeyboardLocation,
            ModifierKey
        };

        pub use crate::webapi::events::progress::{
            IProgressEvent,
            ProgressEvent,
            LoadStartEvent,
            LoadEndEvent,
            ProgressLoadEvent,
            ProgressAbortEvent,
            ProgressErrorEvent
        };

        pub use crate::webapi::events::socket::{
            IMessageEvent,
            SocketCloseEvent,
            SocketErrorEvent,
            SocketOpenEvent,
            SocketMessageEvent,
            SocketMessageData
        };

        pub use crate::webapi::events::history::{
            HashChangeEvent,
            PopStateEvent
        };

        pub use crate::webapi::events::dom::{
            ChangeEvent,
            ResourceLoadEvent,
            ResourceAbortEvent,
            ResourceErrorEvent,
            ResizeEvent,
            ScrollEvent,
            InputEvent,
            ReadyStateChangeEvent,
            SubmitEvent,
            SelectionChangeEvent
        };

        pub use crate::webapi::events::focus::{
            IFocusEvent,
            FocusEvent,
            BlurEvent
        };

        pub use crate::webapi::events::gamepad::{
            IGamepadEvent,
            GamepadConnectedEvent,
            GamepadDisconnectedEvent,
        };

        pub use crate::webapi::events::drag::{
            IDragEvent,
            DragRelatedEvent,
            DragEvent,
            DragStartEvent,
            DragEndEvent,
            DragEnterEvent,
            DragLeaveEvent,
            DragOverEvent,
            DragExitEvent,
            DragDropEvent,
            DataTransfer,
            EffectAllowed,
            DropEffect,
            DataTransferItemList,
            DataTransferItem,
            DataTransferItemKind,
        };

        pub use crate::webapi::events::slot::SlotChangeEvent;
    }

    #[cfg(feature = "experimental_features_which_may_break_on_minor_version_bumps")]
    /// APIs related to MIDI.
    pub mod midi {
        pub use webapi::midi::{
            MidiOptions,
            MidiAccess,
            MidiPort,
            MidiInput,
            MidiOutput,
            IMidiPort
        };
    }
}

/// A module containing stable counterparts to currently
/// unstable Rust features.
pub mod unstable {
    pub use crate::webcore::try_from::{
        TryFrom,
        TryInto
    };

    pub use crate::webcore::void::Void;
}

/// A module containing reexports of all of our interface traits.
///
/// You should **only** import its contents through a wildcard, e.g.: `use stdweb::traits::*`.
pub mod traits {
    #[doc(hidden)]
    pub use super::web::{
        // Real interfaces.
        IEventTarget,
        INode,
        IElement,
        IHtmlElement,
        IBlob,

        // Mixins.
        IWindowOrWorker,
        IParentNode,
        INonElementParentNode,
        IChildNode,
        ISlotable,
    };

    #[doc(hidden)]
    pub use super::web::error::{
        IDomException,
        IError
    };

    #[doc(hidden)]
    pub use super::web::event::{
        IEvent,
        IUiEvent,
        IMouseEvent,
        IPointerEvent,
        IKeyboardEvent,
        IProgressEvent,
        IMessageEvent,
        IFocusEvent,
        IDragEvent,
        ITouchEvent,
    };

    #[cfg(feature = "experimental_features_which_may_break_on_minor_version_bumps")]
    #[doc(hidden)]
    pub use super::web::midi::IMidiPort;
}

#[doc(hidden)]
pub mod private {
    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web)))]
    pub extern crate wasm_bindgen;

    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web)))]
    pub use crate::webcore::ffi::get_module;

    pub use crate::webcore::ffi::exports::*;
    pub use crate::webcore::serialization::{
        JsSerialize,
        JsSerializeOwned,
        SerializedValue
    };

    pub use crate::webcore::newtype::{
        IntoNewtype,
        Newtype
    };

    #[cfg(feature = "serde")]
    pub use crate::ecosystem::serde::{
        to_value,
        from_value
    };

    pub use crate::webcore::global_arena::ArenaRestorePoint;
    pub use crate::webcore::global_arena::serialize_value;

    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", cargo_web))]
    pub use stdweb_internal_macros::wasm32_unknown_unknown_js_attr as js_attr;
    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", cargo_web))]
    pub use stdweb_internal_macros::wasm32_unknown_unknown_js_no_return_attr as js_no_return_attr;
    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", cargo_web))]
    pub use stdweb_internal_macros::wasm32_unknown_unknown_js_raw_attr as js_raw_attr;

    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web)))]
    pub use stdweb_internal_macros::wasm_bindgen_js_attr as js_attr;
    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web)))]
    pub use stdweb_internal_macros::wasm_bindgen_js_no_return_attr as js_no_return_attr;
    #[cfg(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown", not(cargo_web)))]
    pub use stdweb_internal_macros::wasm_bindgen_js_raw_attr as js_raw_attr;

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    pub use stdweb_internal_macros::emscripten_js_attr as js_attr;
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    pub use stdweb_internal_macros::emscripten_js_no_return_attr as js_no_return_attr;
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    pub use stdweb_internal_macros::emscripten_js_raw_attr as js_raw_attr;

    // This is to prevent an unused_mut warnings in macros, because an `allow` doesn't work apparently?
    #[allow(dead_code)]
    #[inline(always)]
    pub fn noop< T >( _: &mut T ) {}

    // TODO: Remove this.
    #[derive(Debug)]
    pub struct TODO;

    impl std::fmt::Display for TODO {
        fn fmt( &self, _: &mut std::fmt::Formatter ) -> Result< (), std::fmt::Error > {
            unreachable!();
        }
    }

    impl std::error::Error for TODO {
        fn description( &self ) -> &str {
            unreachable!();
        }
    }

    pub use crate::webcore::value::ConversionError;
}
