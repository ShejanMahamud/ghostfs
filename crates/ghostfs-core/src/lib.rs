//! # ghostfs-core
//!
//! Core library for GhostFS — dependency virtualization engine.
//! Contains the manifest parser, lockfile, dependency resolver, installer, and linker.

pub mod installer;
pub mod linker;
pub mod lockfile;
pub mod manifest;
pub mod resolver;
pub mod scaffold;

pub use installer::{InstallResult, Installer};
pub use linker::{LinkResult, Linker};
pub use lockfile::{LockedPackage, Lockfile};
pub use manifest::Manifest;
pub use resolver::{DependencyResolver, ResolvedPackage};
pub use scaffold::Scaffolder;
