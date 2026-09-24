#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod command;
pub use command::{
    DrainParams, LoadArgs, LoadMode, LoadTestCli, LoadTestCommand, MaintenanceCommand, RescueArgs,
};
