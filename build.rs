fn main() {
    let target = std::env::var("TARGET").unwrap();
    let abi = match target.as_str() {
        "armv7-linux-androideabi" => "armeabi-v7a",
        "aarch64-linux-android" => "arm64-v8a",
        "i686-linux-android" => "x86",
        "x86_64-unknown-linux-gnu" => return,
        _ => panic!("Unsupported target: {}", target),
    };

    println!("cargo:rustc-link-search=native=./android/app/build/generated/jniLibs/{abi}");
}
