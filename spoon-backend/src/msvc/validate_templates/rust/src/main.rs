unsafe extern "C" {
    fn spoon_validate_helper_value() -> i32;
}

fn main() {
    let values = vec![1_i32, 2, 3];
    let helper = unsafe { spoon_validate_helper_value() };
    println!("sample={{VALIDATE_SAMPLE_LABEL}}");
    println!("rust_runtime=Vec<i32>+fmt ok | values={}", values.len());
    println!("native_helper={{VALIDATE_NATIVE_HELPER_LABEL}} ok | value={}", helper);
    println!("linker_check={{VALIDATE_LINKER_LABEL}} ok");
    assert_eq!(helper, 7);
}
