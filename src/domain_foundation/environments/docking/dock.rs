//! Dock identity, categories, and shared dock information.
//!
//! A dock is one logical unit in the layout (PRD §5): it may represent an
//! individual work area (a panel) or a structural group (split container,
//! tab group, embed host). The environment attaches **no application data**
//! beyond the opaque, host-supplied content reference (PRD §7.3): domain
//! engines layer their own data externally, keyed by [`DockId`].

use super::embed_host::HostRegions;
use super::floating::FloatingEntry;
use super::sizing::SplitSizes;
use std::fmt;

/// Stable, opaque identifier for a dock within a
/// [`DockLayout`](super::model::DockLayout).
///
/// Identifiers are issued by the layout on dock creation and remain valid
/// for the lifetime of the dock within that layout. They are **not** array
/// indices: removing one dock does not renumber the others (PRD DM-01…DM-05).
///
/// External code does not mint `DockId`s for new docks: ids are issued
/// by the layout's creation methods and outcomes (PRD DM-05). The
/// id↔raw mapping (`from_raw`/`as_raw`) is public so host applications
/// can key their content stores against serialized ids and tests can
/// name known-absent ids.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DockId(u64);

impl DockId {
    /// Construct an id from a raw counter value.
    ///
    /// The inverse of [`as_raw`](Self::as_raw): host applications re-key
    /// serialized layout ids against their own content stores, and test
    /// fixtures name known-absent ids with it. The layout's creation
    /// methods remain the source of ids for *new* docks (PRD DM-05:
    /// ids are environment-issued).
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Read the underlying counter value.
    ///
    /// Equal for the same logical dock across save/restore — ids ship
    /// verbatim inside the persisted form (Phase 10) — and stable within
    /// one layout instance; not stable across independently created
    /// layouts (a fresh layout's counter restarts).
    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for DockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dock#{}", self.0)
    }
}

/// Opaque, host-supplied reference to a dock's work-area content
/// (an editor, inspector, canvas, ...).
///
/// Preserved verbatim by the environment; never interpreted by it
/// (PRD §7.2, §7.3, §12.4). `None` reserves the dock's place without
/// naming any content. The host application remains responsible for the
/// actual content lifetime (PRD §8.11).
pub type ContentRef = String;

/// Primary arrangement direction of a split container's children.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitOrientation {
    /// Children arranged horizontally (left to right).
    Row,
    /// Children arranged vertically (top to bottom).
    Column,
}

impl fmt::Display for SplitOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitOrientation::Row => write!(f, "row"),
            SplitOrientation::Column => write!(f, "column"),
        }
    }
}

/// The category (PRD §7.2) a dock belongs to, without its payload.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockKind {
    /// An individual work area.
    Panel,
    /// A split container arranging ordered children along one direction.
    Split,
    /// A container retaining multiple docks with one active member.
    TabGroup,
    /// A dock retaining its own work area plus edge regions and a
    /// floating collection.
    EmbedHost,
}

impl fmt::Display for DockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DockKind::Panel => write!(f, "panel"),
            DockKind::Split => write!(f, "split"),
            DockKind::TabGroup => write!(f, "tab group"),
            DockKind::EmbedHost => write!(f, "embed host"),
        }
    }
}

/// A dock: identity paired with a category payload.
///
/// The environment attaches no application-specific data beyond the
/// host-supplied [`ContentRef`]. Structural collections (`children`,
/// `members`, `regions`, `floating`) are controlled by the
/// [`DockLayout`](super::model::DockLayout); external code reads them
/// through the accessors below.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Dock {
    /// This dock's stable identity.
    pub id: DockId,
    /// This dock's category payload.
    pub role: DockRole,
}

/// Category payloads carried by a [`Dock`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DockRole {
    /// An individual work area. Content is the host-supplied
    /// [`ContentRef`]; the environment does not own or render it.
    ///
    /// `collapsed` is the dock's persistent logical collapse preference
    /// (PRD §9.7): `true` asks the frontend for a collapsed presentation
    /// while the dock keeps its placement, membership, and allocation
    /// exactly as if expanded (see `collapsed()`).
    Panel {
        content: Option<ContentRef>,
        /// Persistent logical collapse preference (`serde(default)`: an
        /// imported layout without the flag restores expanded — the
        /// neutral state).
        #[cfg_attr(feature = "serde", serde(default))]
        collapsed: bool,
    },
    /// An ordered child sequence arranged along one direction.
    /// `sizes` holds one relative weight per child (see [`SplitSizes`]).
    Split {
        orientation: SplitOrientation,
        children: Vec<DockId>,
        sizes: SplitSizes,
    },
    /// Multiple docks sharing one position; `active` indexes `members`
    /// and is valid only while `members` is non-empty.
    TabGroup {
        members: Vec<DockId>,
        active: usize,
    },
    /// Retains its own work area (`content`) while providing edge-region
    /// placements and a floating-dock collection.
    EmbedHost {
        content: Option<ContentRef>,
        regions: HostRegions,
        /// The floating-dock collection: one [`FloatingEntry`] per
        /// floated subtree (PRD §7.5, §8.8 initial baseline), each with
        /// the entry's persistent logical rect (PRD §9.6 — see
        /// [`floating`](super::floating)). Appended last by `Embed`,
        /// one subtree per entry, internal structure preserved.
        floating: Vec<FloatingEntry>,
        /// Where this host stands (PRD §8.9): `true` is an **explicitly
        /// configured persistent host** (e.g. created via
        /// `CreateEmbedHost`) that remains a host even when its regions
        /// and floating collection are all empty — an empty host is a
        /// valid steady state (PRD §8.8). `false` is a **temporary
        /// promotion**: a panel promoted by an embedding operation
        /// ([`PromoteHost`](super::operations::DockingOperation::PromoteHost) or
        /// auto-promoted by `Embed`); when the last embedded/floating
        /// dock leaves, normalization demotes it back to a panel
        /// (identity and content preserved, PRD §8.9 baseline).
        ///
        /// `serde(default = ...)` with `true`: layouts saved before the
        /// persistence distinction restored as persistent hosts (hosts
        /// were always explicit before temporary promotion existed).
        #[cfg_attr(feature = "serde", serde(default = "default_persistent_host"))]
        persistent: bool,
    },
}

/// serde default of `EmbedHost.persistent`: imported hosts without the
/// field are the pre-Phase-7 explicit hosts — persistent (they must not
/// silently demote on import; an intentionally temporary host carries
/// `false` explicitly).
#[cfg(feature = "serde")]
fn default_persistent_host() -> bool {
    true
}

impl Dock {
    /// Create a dock with the given identity and role.
    pub fn new(id: DockId, role: DockRole) -> Self {
        Self { id, role }
    }

    /// This dock's identity.
    pub fn id(&self) -> DockId {
        self.id
    }

    /// This dock's category, without the category payload.
    pub fn kind(&self) -> DockKind {
        match &self.role {
            DockRole::Panel { .. } => DockKind::Panel,
            DockRole::Split { .. } => DockKind::Split,
            DockRole::TabGroup { .. } => DockKind::TabGroup,
            DockRole::EmbedHost { .. } => DockKind::EmbedHost,
        }
    }

    /// The host-supplied content reference for work-area docks
    /// (panels and embed hosts). `None` when the dock is structural
    /// or no content was supplied.
    pub fn content(&self) -> Option<&ContentRef> {
        match &self.role {
            DockRole::Panel { content, .. } | DockRole::EmbedHost { content, .. } => {
                content.as_ref()
            }
            DockRole::Split { .. } | DockRole::TabGroup { .. } => None,
        }
    }

    /// Logical collapse preference of a panel dock, if it is one
    /// (PRD §9.7). Presentation-only for the model: the flag never
    /// changes the dock's placement, membership, or sizing data, so a
    /// collapsed dock remains placed, traversable, and valid — and
    /// restoring it is exactly this flag clearing.
    /// `None` for non-panel docks (v1: panels are the collapsible
    /// category; see `SetDockCollapsed`).
    pub fn collapsed(&self) -> Option<bool> {
        match &self.role {
            DockRole::Panel { collapsed, .. } => Some(*collapsed),
            _ => None,
        }
    }

    /// Whether an embed host is a persistent host (`true`: remains a
    /// host when empty) or a temporary promotion (`false`: demoted back
    /// to a panel by normalization when its last embedded/floating dock
    /// leaves — PRD §8.9). `None` for non-host docks.
    pub fn persistent(&self) -> Option<bool> {
        match &self.role {
            DockRole::EmbedHost { persistent, .. } => Some(*persistent),
            _ => None,
        }
    }

    /// Child order of a split dock, if it is one.
    pub fn children(&self) -> Option<&[DockId]> {
        match &self.role {
            DockRole::Split { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Orientation of a split dock, if it is one.
    pub fn orientation(&self) -> Option<SplitOrientation> {
        match &self.role {
            DockRole::Split { orientation, .. } => Some(*orientation),
            _ => None,
        }
    }

    /// Relative sizes of a split dock's children, if it is one.
    pub fn sizes(&self) -> Option<&SplitSizes> {
        match &self.role {
            DockRole::Split { sizes, .. } => Some(sizes),
            _ => None,
        }
    }

    /// Active-tab index of a tab group, if it is one.
    /// Valid only while the group has members.
    pub fn active(&self) -> Option<usize> {
        match &self.role {
            DockRole::TabGroup { active, .. } => Some(*active),
            _ => None,
        }
    }

    /// Occupied edge regions of an embed host, if it is one.
    pub fn host_regions(&self) -> Option<&HostRegions> {
        match &self.role {
            DockRole::EmbedHost { regions, .. } => Some(regions),
            _ => None,
        }
    }

    /// Floating-dock collection of an embed host, if it is one — one
    /// [`FloatingEntry`] per floated subtree, each carrying its
    /// persistent logical rect (PRD §9.6).
    pub fn floating(&self) -> Option<&[FloatingEntry]> {
        match &self.role {
            DockRole::EmbedHost { floating, .. } => Some(floating),
            _ => None,
        }
    }

    /// Structural children of any dock that can own children:
    /// split children, tab members, then — for hosts — occupied edge
    /// regions followed by floating entries (PRD §7.5).
    pub fn structural_children(&self) -> Vec<DockId> {
        match &self.role {
            DockRole::Split { children, .. } => children.clone(),
            DockRole::TabGroup { members, .. } => members.clone(),
            DockRole::EmbedHost { regions, floating, .. } => regions
                .occupied_docks()
                .into_iter()
                .chain(floating.iter().map(|entry| entry.dock))
                .collect(),
            DockRole::Panel { .. } => Vec::new(),
        }
    }
}

impl fmt::Display for Dock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.id, self.kind())
    }
}
