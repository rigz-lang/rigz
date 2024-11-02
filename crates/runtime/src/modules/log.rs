use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(r#"
trait Log
    # `log :level, template, args` is built in but doesn't support identifiers yet

    fn info(template: String, var args) -> None
        log :info, template, args
    end

    fn warn(template: String, var args) -> None
        log :warn, template, args
    end

    fn trace(template: String, var args) -> None
        log :info, template, args
    end

    fn debug(template: String, var args) -> None
        log :debug, template, args
    end

    fn error(template: String, var args) -> None
      log :error, template, args
    end
end
"#);

impl RigzLog for LogModule {}