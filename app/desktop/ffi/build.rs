fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("libfd")
        .csharp_class_name("Native")
        .generate_csharp_file("../win/FlyDrop/FlyDrop.Core/Native.g.cs")
        .unwrap();
}
