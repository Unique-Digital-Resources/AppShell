//! `PluginPermissions` — bitflags for what a plugin is allowed to do.

/// Permission flags controlling what a plugin can access.
///
/// Declared in the manifest and enforced by the PluginManager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluginPermissions {
    bits: u32,
}

pub const REGISTER_COMMANDS: u32 = 0b0000_0001;
pub const REGISTER_TASKS: u32 = 0b0000_0010;
pub const LOAD_RESOURCES: u32 = 0b0000_0100;
pub const REGISTER_SERVICES: u32 = 0b0000_1000;
pub const SUBSCRIBE_EVENTS: u32 = 0b0001_0000;
pub const PUBLISH_EVENTS: u32 = 0b0010_0000;
pub const READ_CONFIG: u32 = 0b0100_0000;
pub const WRITE_CONFIG: u32 = 0b1000_0000;
pub const FULL_ACCESS: u32 = 0b1111_1111;

impl PluginPermissions {
    pub const NONE: PluginPermissions = PluginPermissions { bits: 0 };
    pub const FULL: PluginPermissions = PluginPermissions { bits: FULL_ACCESS };

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn bits(&self) -> u32 {
        self.bits
    }

    pub fn has(&self, permission: u32) -> bool {
        (self.bits & permission) == permission
    }

    pub fn can_register_commands(&self) -> bool {
        self.has(REGISTER_COMMANDS)
    }

    pub fn can_register_tasks(&self) -> bool {
        self.has(REGISTER_TASKS)
    }

    pub fn can_load_resources(&self) -> bool {
        self.has(LOAD_RESOURCES)
    }

    pub fn can_register_services(&self) -> bool {
        self.has(REGISTER_SERVICES)
    }

    pub fn can_subscribe_events(&self) -> bool {
        self.has(SUBSCRIBE_EVENTS)
    }

    pub fn can_publish_events(&self) -> bool {
        self.has(PUBLISH_EVENTS)
    }

    pub fn can_read_config(&self) -> bool {
        self.has(READ_CONFIG)
    }

    pub fn can_write_config(&self) -> bool {
        self.has(WRITE_CONFIG)
    }
}

impl std::ops::BitOr for PluginPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl std::ops::BitOr<u32> for PluginPermissions {
    type Output = Self;

    fn bitor(self, rhs: u32) -> Self {
        Self {
            bits: self.bits | rhs,
        }
    }
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self::FULL // Default: full access (backward compatible)
    }
}

impl std::fmt::Display for PluginPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginPermissions({:08b})", self.bits)
    }
}