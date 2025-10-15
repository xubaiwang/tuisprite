use std::{cell::RefCell, rc::Rc, sync::mpsc};

use andromeda_core::{HostData, RuntimeHostHooks};
use andromeda_runtime::RuntimeMacroTask;
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, BuiltinFunctionArgs, create_builtin_function},
        execution::{
            Agent, JsResult,
            agent::{GcAgent, Options, RealmRoot},
        },
        scripts_and_modules::script::{parse_script, script_evaluation},
        types::{
            self, InternalMethods, IntoFunction, Object, PropertyDescriptor, PropertyKey, Value,
        },
    },
    engine::context::{Bindable, GcScope},
};

use crate::app::config::Config;

struct AppResource {
    config: Rc<RefCell<Config>>,
}

/// The JavaScript script execution runtime.
pub struct Runtime {
    agent: GcAgent,
    realm: RealmRoot,
}

impl Runtime {
    pub fn new(config: Rc<RefCell<Config>>) -> Self {
        let (_macro_task_tx, _macro_task_rx) = mpsc::channel();
        let host_data = HostData::new(_macro_task_tx);

        {
            let mut map = host_data.storage.borrow_mut();
            map.insert(AppResource { config });
        }

        let host_hooks = RuntimeHostHooks::new(host_data);
        let host_hooks: &RuntimeHostHooks<RuntimeMacroTask> = &*Box::leak(Box::new(host_hooks));

        let mut agent = GcAgent::new(
            Options {
                disable_gc: false,
                print_internals: false,
            },
            host_hooks,
        );

        let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
            None;
        let create_global_this_value: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let realm = agent.create_realm(
            create_global_object,
            create_global_this_value,
            Some(
                |agent: &mut Agent, global_object: Object<'_>, mut gc: GcScope<'_, '_>| {
                    // builtin
                    let function = create_builtin_function(
                        agent,
                        nova_vm::ecmascript::builtins::Behaviour::Regular(color_setter),
                        BuiltinFunctionArgs::new(1, "_set_color"),
                        gc.nogc(),
                    );

                    let property_key = PropertyKey::from_static_str(agent, "color", gc.nogc());
                    global_object
                        .internal_define_own_property(
                            agent,
                            property_key.unbind(),
                            PropertyDescriptor {
                                set: Some(Some(function.into_function().unbind())),
                                ..Default::default()
                            },
                            gc.reborrow(),
                        )
                        .unwrap();
                },
            ),
        );

        Self { agent, realm }
    }

    pub fn execute_script(&mut self, script: &str) -> anyhow::Result<()> {
        self.agent
            .run_in_realm(&self.realm, |agent, mut gc| -> anyhow::Result<()> {
                let realm_obj = agent.current_realm(gc.nogc());
                let source_text = types::String::from_str(agent, script, gc.nogc());

                let script =
                    parse_script(agent, source_text, realm_obj, true, None, gc.nogc()).unwrap();

                let _result = script_evaluation(agent, script.unbind(), gc.reborrow()).unbind();

                Ok(())
            })?;
        Ok(())
    }
}

fn color_setter<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let color = args.get(0).to_string(agent, gc.reborrow()).unbind()?;

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res = storage.get_mut::<AppResource>().unwrap();

    if let Ok(color) = csscolorparser::Color::from_html(color.to_string_lossy(agent)) {
        res.config.borrow_mut().set_color(color);
    };
    // TODO: report error info

    Ok(Value::Undefined)
}
