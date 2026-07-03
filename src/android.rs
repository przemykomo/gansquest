use gansui::sdl3;
use jni::{EnvUnowned, JValue, JValueOwned, objects::JString};

pub fn create_subdirectory(parent: &str, subdir: &str) -> String {
    unsafe {
        let ptr: *mut ::core::ffi::c_void = sdl3::sys::system::SDL_GetAndroidJNIEnv();
        let mut env = jni::EnvUnowned::from_raw(ptr as _);
        let res = env.with_env(|env| -> Result<_, jni::errors::Error> {
            let parent = JString::new(env, parent)?.into();
            let dir = JString::new(env, subdir)?.into();
            let res = env.call_static_method(
                jni::jni_str!("przemyk/gansquest/MyActivity"),
                jni::jni_str!("createDirectory"),
                jni::jni_sig!(sig = (arg1: java.lang.String, arg2: java.lang.String) -> java.lang.String),
                    & [JValue::Object(
                        &parent,
                    ), JValue::Object(
                        &dir
                    )],
            ).unwrap();

            let JValueOwned::Object(obj) = res else { panic!() };
            JString::cast_local(env, obj)?.try_to_string(env)
        });

        let res = res.resolve::<jni::errors::LogErrorAndDefault>();
        res.into()
    }
}
