/// Prost-encoded wire representation of a portable view tree.
///
/// `BrickViewBlueprint` is built once at component mount time and sent to the
/// native host (Kotlin/Android, Swift/iOS) as a `jbyteArray` over JNI.
/// Signal subscriptions and action dispatching are wired separately by
/// `src/native/mod.rs` after the blueprint is delivered.
///
/// The blueprint is intentionally static — reactive updates flow through the
/// drain queue (`SIGNAL_QUEUE`), not through blueprint re-sends.
use crate::portable::{BlueprintNode, PortableContent, PortableView};

// ── Prost message types ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq, prost::Message)]
pub struct BrickViewBlueprint {
    /// Root node of the view tree.
    #[prost(message, optional, tag = "1")]
    pub root: Option<ProtoNode>,

    /// All signals referenced anywhere in the tree, in registration order.
    #[prost(message, repeated, tag = "2")]
    pub signals: Vec<SignalDescriptor>,

    /// All controller actions referenced anywhere in the tree.
    #[prost(message, repeated, tag = "3")]
    pub actions: Vec<ActionDescriptor>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct SignalDescriptor {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    /// UTF-8 string representation of the signal's current value.
    #[prost(string, tag = "2")]
    pub initial_value: String,
    /// "string" | "i32" | "f64" | "bool" — hint for the native renderer.
    #[prost(string, tag = "3")]
    pub value_type: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ActionDescriptor {
    /// Matches the `BrickAction::name` on the Rust side.
    #[prost(string, tag = "1")]
    pub name: String,
    /// "click" | "input" | "submit" — the originating event type.
    #[prost(string, tag = "2")]
    pub event: String,
}

// ── ProtoNode — the oneof that encodes BlueprintNode ─────────────────────────

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoNode {
    #[prost(
        oneof = "proto_node::Kind",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9"
    )]
    pub kind: Option<proto_node::Kind>,
}

pub mod proto_node {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Kind {
        #[prost(message, tag = "1")]
        Text(super::ProtoText),
        #[prost(message, tag = "2")]
        Button(super::ProtoButton),
        #[prost(message, tag = "3")]
        Input(super::ProtoInput),
        #[prost(message, tag = "4")]
        Column(super::ProtoContainer),
        #[prost(message, tag = "5")]
        Row(super::ProtoContainer),
        #[prost(message, tag = "6")]
        Scroll(super::ProtoContainer),
        #[prost(message, tag = "7")]
        Toggle(super::ProtoToggle),
        #[prost(message, tag = "8")]
        ExternalIntent(super::ProtoExternalIntent),
        #[prost(message, tag = "9")]
        PermissionRequest(super::ProtoPermissionRequest),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoText {
    /// Non-empty for static text; empty when `signal_id` is set.
    #[prost(string, tag = "1")]
    pub static_text: String,
    /// Non-zero signals that this text is reactive; maps to a `SignalDescriptor`.
    #[prost(uint32, tag = "2")]
    pub signal_id: u32,
    /// 0=Body 1=Title 2=Caption 3=Label — maps to MaterialTheme.typography on Android.
    #[prost(int32, tag = "3")]
    pub text_variant: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoButton {
    #[prost(string, tag = "1")]
    pub label: String,
    /// Empty string means no action bound.
    #[prost(string, tag = "2")]
    pub action: String,
    /// Method name for index-based dispatch (`dispatchInt`). Empty = not used.
    #[prost(string, tag = "3")]
    pub int_action: String,
    /// Index payload sent with `dispatchInt` when `int_action` is set.
    #[prost(int32, tag = "4")]
    pub payload_int: i32,
    /// 0=Primary 1=Secondary 2=Outlined 3=Ghost 4=Danger — maps to M3 composables on Android.
    #[prost(int32, tag = "5")]
    pub variant: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoToggle {
    #[prost(string, tag = "1")]
    pub label: String,
    /// 0 means no two-way binding.
    #[prost(uint32, tag = "2")]
    pub checked_signal_id: u32,
    /// Empty string means no change action.
    #[prost(string, tag = "3")]
    pub change_action: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoExternalIntent {
    #[prost(string, tag = "1")]
    pub label: String,
    /// Discriminant: 0=ShareText 1=OpenUrl 2=LaunchCamera 3=PickImage 4=OpenSettings 5=ComposeEmail
    #[prost(int32, tag = "2")]
    pub intent_type: i32,
    /// Primary string payload: text (Share), url (OpenUrl), result_action (Camera/Image), email "to" (ComposeEmail).
    #[prost(string, tag = "3")]
    pub primary_payload: String,
    /// Secondary payload: email subject for ComposeEmail.
    #[prost(string, tag = "4")]
    pub secondary_payload: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoPermissionRequest {
    #[prost(string, tag = "1")]
    pub label: String,
    /// Android permission string e.g. "android.permission.CAMERA".
    #[prost(string, tag = "2")]
    pub permission: String,
    /// Controller action called with `dispatchBool(result_action, isGranted)`.
    #[prost(string, tag = "3")]
    pub result_action: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoInput {
    #[prost(string, tag = "1")]
    pub placeholder: String,
    /// 0 means no two-way binding.
    #[prost(uint32, tag = "2")]
    pub value_signal_id: u32,
    /// Empty string means no change action.
    #[prost(string, tag = "3")]
    pub change_action: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoContainer {
    #[prost(message, repeated, tag = "1")]
    pub children: Vec<ProtoNode>,
}

// ── Conversion: BlueprintNode → ProtoNode ────────────────────────────────────

impl From<BlueprintNode> for ProtoNode {
    fn from(node: BlueprintNode) -> Self {
        use crate::portable::IntentType;
        use proto_node::Kind;
        let kind = match node {
            BlueprintNode::Text { content, variant } => Kind::Text(ProtoText {
                text_variant: variant as i32,
                ..ProtoText::from(content)
            }),
            BlueprintNode::Button { label, action, int_action, int_payload, variant } => {
                Kind::Button(ProtoButton {
                    label,
                    action: action.unwrap_or_default(),
                    int_action: int_action.unwrap_or_default(),
                    payload_int: int_payload.unwrap_or(0),
                    variant: variant as i32,
                })
            }
            BlueprintNode::Input { placeholder, value_signal_id, change_action } => {
                Kind::Input(ProtoInput {
                    placeholder,
                    value_signal_id: value_signal_id.unwrap_or(0),
                    change_action: change_action.unwrap_or_default(),
                })
            }
            BlueprintNode::Column(children) => Kind::Column(ProtoContainer {
                children: children.into_iter().map(ProtoNode::from).collect(),
            }),
            BlueprintNode::Row(children) => Kind::Row(ProtoContainer {
                children: children.into_iter().map(ProtoNode::from).collect(),
            }),
            BlueprintNode::Scroll(children) => Kind::Scroll(ProtoContainer {
                children: children.into_iter().map(ProtoNode::from).collect(),
            }),
            BlueprintNode::Toggle { label, checked_signal_id, change_action } => {
                Kind::Toggle(ProtoToggle {
                    label,
                    checked_signal_id: checked_signal_id.unwrap_or(0),
                    change_action: change_action.unwrap_or_default(),
                })
            }
            BlueprintNode::ExternalIntent { label, intent_type } => {
                let (intent_type_tag, primary, secondary) = match intent_type {
                    IntentType::ShareText { text } => (0, text, String::new()),
                    IntentType::OpenUrl { url } => (1, url, String::new()),
                    IntentType::LaunchCamera { result_action } => (2, result_action, String::new()),
                    IntentType::PickImage { result_action } => (3, result_action, String::new()),
                    IntentType::OpenSettings => (4, String::new(), String::new()),
                    IntentType::ComposeEmail { to, subject } => (5, to, subject),
                };
                Kind::ExternalIntent(ProtoExternalIntent {
                    label,
                    intent_type: intent_type_tag,
                    primary_payload: primary,
                    secondary_payload: secondary,
                })
            }
            BlueprintNode::PermissionRequest { label, permission, result_action } => {
                Kind::PermissionRequest(ProtoPermissionRequest { label, permission, result_action })
            }
        };
        ProtoNode { kind: Some(kind) }
    }
}

impl From<PortableContent> for ProtoText {
    fn from(content: PortableContent) -> Self {
        match content {
            PortableContent::Static(s) => ProtoText { static_text: s, signal_id: 0, text_variant: 0 },
            PortableContent::Reactive { signal_id, .. } => {
                ProtoText { static_text: String::new(), signal_id, text_variant: 0 }
            }
        }
    }
}

// ── Blueprint builder ─────────────────────────────────────────────────────────

/// Encode a `PortableView` tree as a prost-encoded `BrickViewBlueprint`.
/// The encoded bytes are passed to the native host via JNI as `jbyteArray`.
pub fn encode_blueprint(view: &dyn PortableView) -> Vec<u8> {
    use prost::Message as _;
    let blueprint = BrickViewBlueprint {
        root: Some(ProtoNode::from(view.blueprint_node())),
        signals: Vec::new(), // populated by native layer from SIGNAL_QUEUE metadata
        actions: Vec::new(), // populated by native layer from controller registry
    };
    blueprint.encode_to_vec()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portable::{button, column, into_portable, p};
    use crate::state_mgmt::{BrickAction, Signal};
    use prost::Message as _;

    #[test]
    fn encodes_and_decodes_static_text() {
        let view = p("hello");
        let bytes = encode_blueprint(&*view);
        let decoded = BrickViewBlueprint::decode(bytes.as_slice()).unwrap();
        match decoded.root.unwrap().kind.unwrap() {
            proto_node::Kind::Text(t) => assert_eq!(t.static_text, "hello"),
            _ => panic!("expected text node"),
        }
    }

    #[test]
    fn encodes_reactive_text_with_signal_id() {
        let sig = Signal::new(99i32);
        let view = p(&sig);
        let bytes = encode_blueprint(&*view);
        let decoded = BrickViewBlueprint::decode(bytes.as_slice()).unwrap();
        match decoded.root.unwrap().kind.unwrap() {
            proto_node::Kind::Text(t) => {
                assert_ne!(t.signal_id, 0, "reactive text must carry a signal_id");
                assert!(t.static_text.is_empty());
            }
            _ => panic!("expected text node"),
        }
    }

    #[test]
    fn encodes_button_with_action() {
        let action = BrickAction { name: "increment", event: "click" };
        let view = button("Click").trigger(&action);
        let bytes = encode_blueprint(&*view);
        let decoded = BrickViewBlueprint::decode(bytes.as_slice()).unwrap();
        match decoded.root.unwrap().kind.unwrap() {
            proto_node::Kind::Button(b) => {
                assert_eq!(b.label, "Click");
                assert_eq!(b.action, "increment");
            }
            _ => panic!("expected button node"),
        }
    }

    #[test]
    fn encodes_column_with_children() {
        let view = column(vec![into_portable(p("a")), into_portable(p("b"))]);
        let bytes = encode_blueprint(&*view);
        let decoded = BrickViewBlueprint::decode(bytes.as_slice()).unwrap();
        match decoded.root.unwrap().kind.unwrap() {
            proto_node::Kind::Column(c) => assert_eq!(c.children.len(), 2),
            _ => panic!("expected column node"),
        }
    }

    #[test]
    fn round_trip_preserves_nested_structure() {
        let view = column(vec![
            into_portable(p("label")),
            into_portable(button("Go").trigger(&BrickAction { name: "go", event: "click" })),
        ]);
        let bytes = encode_blueprint(&*view);
        assert!(!bytes.is_empty());
        let decoded = BrickViewBlueprint::decode(bytes.as_slice()).unwrap();
        match decoded.root.unwrap().kind.unwrap() {
            proto_node::Kind::Column(c) => {
                assert_eq!(c.children.len(), 2);
                assert!(matches!(
                    c.children[0].kind.as_ref().unwrap(),
                    proto_node::Kind::Text(_)
                ));
                assert!(matches!(
                    c.children[1].kind.as_ref().unwrap(),
                    proto_node::Kind::Button(_)
                ));
            }
            _ => panic!("expected column"),
        }
    }
}
