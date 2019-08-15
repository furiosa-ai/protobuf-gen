use failure::Fallible;

use protobuf_gen::Config;

#[test]
fn unittest_yellow_book() -> Fallible<()> {
    env_logger::try_init().unwrap_or_default();

    let mut config = Config::new("protos", Some("proxy"));
    config.add_source("lib_tests/src/person.rs", "yellow_book");
    config.add_source("lib_tests/src/city.rs", "yellow_book");

    config.generate()?;
    Ok(())
}

#[test]
fn unittest_npu_ir() -> Fallible<()> {
    env_logger::try_init().unwrap_or_default();

    let mut config = Config::new("protos", Some("proxy"));
    config.replace_type("TensorIndex".to_string(), "uint32".to_string());
    config.replace_type("AxisIndex".to_string(), "uint32".to_string());
    config.replace_type("Tag".to_string(), "uint32".to_string());
    config.replace_type("MemorySize_Io".to_string(), "uint32".to_string());

    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/common/mod.rs",
        "npu_ir.common",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/common/tensor.rs",
        "npu_ir.common",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/common/element_type.rs",
        "npu_ir.common",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/common/shape.rs",
        "npu_ir.common",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/common/operator_group.rs",
        "npu_ir.common",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/common/operator/mod.rs",
        "npu_ir.common",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/dfg/mod.rs",
        "npu_ir.dfg",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/ldfg/mod.rs",
        "npu_ir.ldfg",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/cdfg/mod.rs",
        "npu_ir.cdfg",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/gir/mod.rs",
        "npu_ir.gir",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/lir/instruction.rs",
        "npu_ir.lir",
    );
    config.add_source(
        "/home/comatose/workspace/npu-tools/crates/npu-ir/src/lir/mod.rs",
        "npu_ir.lir",
    );

    config.generate()?;
    Ok(())
}
