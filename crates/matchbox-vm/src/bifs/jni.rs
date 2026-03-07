#[cfg(not(target_arch = "wasm32"))]
use jni::{
    objects::{GlobalRef, JObject, JValue},
    InitArgsBuilder, JNIVersion, JavaVM,
};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, OnceLock};

use crate::types::BxValue;

#[cfg(not(target_arch = "wasm32"))]
use crate::types::{BxNativeObject, BxVM, Constant};
use crate::vm::gc::GcObject;
#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
static JVM: OnceLock<Result<Arc<JavaVM>, String>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn get_jvm() -> Result<Arc<JavaVM>, String> {
    let res = JVM.get_or_init(|| {
        let jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .build()
            .map_err(|e| format!("Failed to build JVM args: {}", e))?;
        let jvm = JavaVM::new(jvm_args).map_err(|e| format!("Failed to create JVM: {}", e))?;
        Ok(Arc::new(jvm))
    });
    res.clone()
}

pub fn create_java_object(vm: &mut dyn BxVM, #[allow(unused_variables)] class_name: &str) -> Result<BxValue, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return Err("Java interoperability is not supported in WASM environments.".to_string());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let jvm = get_jvm()?;
        let mut env = jvm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
        
        // Convert class name from "java.util.ArrayList" to "java/util/ArrayList"
        let jni_class_name = class_name.replace(".", "/");
        
        let obj = env.new_object(&jni_class_name, "()V", &[])
            .map_err(|e| format!("Failed to instantiate {}: {}", class_name, e))?;
        
        let global_ref = env.new_global_ref(obj)
            .map_err(|e| format!("Failed to create global ref: {}", e))?;
            
        let id = vm.string_new("java_object".to_string()); // dummy for now, should use NativeObject variant properly
        // NativeObject needs to be on GC heap. BxVM should have a way to alloc it.
        // For now I'll just return null or error until I add native object support to BxVM trait
        Err("Java JNI support needs BxVM to support NativeObject allocation".to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct JniObject {
    jvm: Arc<JavaVM>,
    global_ref: GlobalRef,
}

#[cfg(not(target_arch = "wasm32"))]
impl BxNativeObject for JniObject {
    fn get_property(&self, _name: &str) -> BxValue {
        BxValue::new_null()
    }

    fn set_property(&mut self, _name: &str, _value: BxValue) {
    }

    fn call_method(&mut self, vm: &mut dyn BxVM, name: &str, args: &[BxValue]) -> Result<BxValue, String> {
        let mut env = self.jvm.attach_current_thread()
            .map_err(|e| format!("Failed to attach thread: {}", e))?;
            
        let obj = self.global_ref.as_obj();
        let class = env.get_object_class(obj).map_err(|e| format!("Failed to get class: {}", e))?;

        // Special handling for constructors via .init()
        if name.to_lowercase() == "init" {
            let get_constructors = env.call_method(&class, "getConstructors", "()[Ljava/lang/reflect/Constructor;", &[])
                .map_err(|e| format!("Failed to get constructors: {}", e))?
                .l().map_err(|e| format!("Invalid return type: {}", e))?;
            
            let constructors_array: &jni::objects::JObjectArray = (&get_constructors).into();
            let constructor_count = env.get_array_length(constructors_array).map_err(|e| format!("Failed to get array length: {}", e))?;

            for i in 0..constructor_count {
                let constructor_obj = env.get_object_array_element(constructors_array, i).map_err(|e| format!("Failed to get array element: {}", e))?;
                
                let parameter_types_val = env.call_method(&constructor_obj, "getParameterTypes", "()[Ljava/lang/Class;", &[])
                    .map_err(|e| format!("Failed to get parameter types: {}", e))?
                    .l().map_err(|e| format!("Invalid return type: {}", e))?;
                let parameter_types_array: &jni::objects::JObjectArray = (&parameter_types_val).into();
                let param_count = env.get_array_length(parameter_types_array).map_err(|e| format!("Failed to get array length: {}", e))?;

                if param_count as usize == args.len() {
                    let mut compatible = true;
                    for (idx, arg) in args.iter().enumerate() {
                        let param_type_obj = env.get_object_array_element(parameter_types_array, idx as i32).map_err(|e| format!("Failed to get parameter type: {}", e))?;
                        let param_class: &jni::objects::JClass = (&param_type_obj).into();
                        
                        let arg_class_name = if arg.is_ptr() { "java/lang/String" } // assumes string for now
                                            else if arg.is_number() { "java/lang/Double" }
                                            else if arg.is_bool() { "java/lang/Boolean" }
                                            else { "java/lang/Object" };

                        let arg_class = env.find_class(arg_class_name).map_err(|e| format!("Class not found: {}", e))?;
                        
                        let is_assignable = env.call_method(param_class, "isAssignableFrom", "(Ljava/lang/Class;)Z", &[JValue::from(&arg_class)])
                            .map_err(|e| format!("Failed to call isAssignableFrom: {}", e))?
                            .z().map_err(|e| format!("Invalid return type: {}", e))?;
                        
                        if !is_assignable {
                            compatible = false;
                            break;
                        }
                    }

                    if !compatible { continue; }

                    let object_class = env.find_class("java/lang/Object").map_err(|e| format!("Class not found: {}", e))?;
                    let args_array = env.new_object_array(args.len() as i32, object_class, JObject::null())
                        .map_err(|e| format!("Failed to create args array: {}", e))?;
                    
                    let mut jobjs = Vec::new();
                    for arg in args {
                        let jobj = if arg.is_ptr() {
                            let s = vm.to_string(*arg);
                            JObject::from(env.new_string(s).map_err(|e| format!("Failed to create string: {}", e))?)
                        } else if arg.is_number() {
                            let double_class = env.find_class("java/lang/Double").map_err(|e| format!("Class not found: {}", e))?;
                            env.new_object(double_class, "(D)V", &[JValue::from(arg.as_number())]).map_err(|e| format!("Failed to wrap double: {}", e))?
                        } else if arg.is_bool() {
                            let boolean_class = env.find_class("java/lang/Boolean").map_err(|e| format!("Class not found: {}", e))?;
                            env.new_object(boolean_class, "(Z)V", &[JValue::from(arg.as_bool())]).map_err(|e| format!("Failed to wrap boolean: {}", e))?
                        } else {
                            return Err(format!("Unsupported argument type for JNI constructor: {:?}", arg));
                        };
                        jobjs.push(jobj);
                    }

                    let jargs: Vec<JValue> = jobjs.iter().map(|obj| JValue::Object(obj)).collect();
                    for (idx, jarg) in jargs.iter().enumerate() {
                        env.set_object_array_element(&args_array, idx as i32, jarg.l().map_err(|e| format!("Arg is not an object: {}", e))?)
                            .map_err(|e| format!("Failed to set array element: {}", e))?;
                    }

                    let new_instance = env.call_method(&constructor_obj, "newInstance", "([Ljava/lang/Object;)Ljava/lang/Object;", 
                        &[JValue::Object(&args_array)])
                        .map_err(|e| format!("Failed to invoke constructor: {}", e))?
                        .l().map_err(|e| format!("Constructor returned invalid type: {}", e))?;

                    // Return native object - needs VM help to alloc
                    return Err("Returning native objects from JNI needs VM support".to_string());
                }
            }
            return Err(format!("Constructor with {} arguments not found on class", args.len()));
        }

        let mut jobjs = Vec::new();
        for arg in args {
            if arg.is_ptr() {
                let s = vm.to_string(*arg);
                jobjs.push(JObject::from(env.new_string(s).map_err(|e| format!("Failed to create string: {}", e))?));
            } else if arg.is_number() {
                let double_class = env.find_class("java/lang/Double").map_err(|e| format!("Class not found: {}", e))?;
                jobjs.push(env.new_object(double_class, "(D)V", &[JValue::from(arg.as_number())]).map_err(|e| format!("Failed to wrap double: {}", e))?);
            } else if arg.is_bool() {
                let boolean_class = env.find_class("java/lang/Boolean").map_err(|e| format!("Class not found: {}", e))?;
                jobjs.push(env.new_object(boolean_class, "(Z)V", &[JValue::from(arg.as_bool())]).map_err(|e| format!("Failed to wrap boolean: {}", e))?);
            } else {
                return Err(format!("Unsupported argument type for JNI: {:?}", arg));
            }
        }
        let jargs: Vec<JValue> = jobjs.iter().map(|obj| JValue::Object(obj)).collect();

        let get_methods = env.call_method(&class, "getMethods", "()[Ljava/lang/reflect/Method;", &[])
            .map_err(|e| format!("Failed to get methods: {}", e))?
            .l().map_err(|e| format!("Invalid return type: {}", e))?;
        
        let methods_array: &jni::objects::JObjectArray = (&get_methods).into();
        let method_count = env.get_array_length(methods_array).map_err(|e| format!("Failed to get array length: {}", e))?;

        for i in 0..method_count {
            let method_obj = env.get_object_array_element(methods_array, i).map_err(|e| format!("Failed to get array element: {}", e))?;
            let name_val = env.call_method(&method_obj, "getName", "()Ljava/lang/String;", &[]).map_err(|e| format!("Failed to get name: {}", e))?.l().unwrap();
            let rname: String = env.get_string((&name_val).into()).map_err(|e| format!("Failed to get string: {}", e))?.into();

            if rname.to_lowercase() == name.to_lowercase() {
                let parameter_types_val = env.call_method(&method_obj, "getParameterTypes", "()[Ljava/lang/Class;", &[]).map_err(|e| format!("Failed to get params: {}", e))?.l().unwrap();
                let parameter_types_array: &jni::objects::JObjectArray = (&parameter_types_val).into();
                let param_count = env.get_array_length(parameter_types_array).map_err(|e| format!("Failed to get array length: {}", e))?;

                if param_count as usize == args.len() {
                    let mut compatible = true;
                    for (idx, arg) in args.iter().enumerate() {
                        let param_type_obj = env.get_object_array_element(parameter_types_array, idx as i32).map_err(|e| format!("Failed to get parameter type: {}", e))?;
                        let param_class: &jni::objects::JClass = (&param_type_obj).into();
                        let arg_class_name = if arg.is_ptr() { "java/lang/String" } else if arg.is_number() { "java/lang/Double" } else if arg.is_bool() { "java/lang/Boolean" } else { "java/lang/Object" };
                        let arg_class = env.find_class(arg_class_name).map_err(|e| format!("Class not found: {}", e))?;
                        let is_assignable = env.call_method(param_class, "isAssignableFrom", "(Ljava/lang/Class;)Z", &[JValue::from(&arg_class)]).map_err(|e| format!("Failed call: {}", e))?.z().unwrap();
                        if !is_assignable { compatible = false; break; }
                    }

                    if !compatible { continue; }

                    let object_class = env.find_class("java/lang/Object").map_err(|e| format!("Class not found: {}", e))?;
                    let args_array = env.new_object_array(args.len() as i32, object_class, JObject::null())
                        .map_err(|e| format!("Failed to create args array: {}", e))?;

                    let result_obj = env.call_method(&method_obj, "invoke", "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;", 
                        &[JValue::Object(&obj), JValue::Object(&args_array)])
                        .map_err(|e| format!("Failed to invoke method {}: {}", name, e))?
                        .l().map_err(|e| format!("Invoke returned invalid type: {}", e))?;

                    if result_obj.is_null() { return Ok(BxValue::new_null()); }

                    let res_class = env.get_object_class(&result_obj).unwrap();
                    let res_class_name_val = env.call_method(&res_class, "getName", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
                    let res_class_name: String = env.get_string((&res_class_name_val).into()).unwrap().into();

                    return match res_class_name.as_str() {
                        "java.lang.String" => {
                            let s: String = env.get_string((&result_obj).into()).unwrap().into();
                            Ok(BxValue::new_ptr(vm.string_new(s)))
                        }
                        "java.lang.Double" | "java.lang.Float" | "java.lang.Integer" | "java.lang.Long" => {
                            let d = env.call_method(&result_obj, "doubleValue", "()D", &[]).unwrap().d().unwrap();
                            Ok(BxValue::new_number(d))
                        }
                        "java.lang.Boolean" => {
                            let b = env.call_method(&result_obj, "booleanValue", "()Z", &[]).unwrap().z().unwrap();
                            Ok(BxValue::new_bool(b))
                        }
                        _ => Err("Returning native objects from JNI needs VM support".to_string()),
                    };
                }
            }
        }
        Err(format!("Method {} with {} arguments not found", name, args.len()))
    }
}
