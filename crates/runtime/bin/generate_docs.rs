use rigz_core::{Definition, GenDocs};
use rigz_runtime::{
    AnyModule, AssertionsModule, CollectionsModule, DateModule, FileModule, HtmlModule, HttpModule,
    JSONModule, NumberModule, Random, StringModule, UUID,
};
use std::io;

macro_rules! saved {
    ($res: ident: $($base: ident)*) => {
        $(
          $res($base::name(), $base::generate_docs())?;
        )*
    };
}
fn main() -> io::Result<()> {
    std::fs::create_dir_all("docs")?;
    let to_save =
        |name: &str, value| std::fs::write(format!("docs/{}.md", name.to_lowercase()), value);
    saved! {
        to_save:
        AnyModule
        AssertionsModule
        NumberModule
        StringModule
        CollectionsModule
        JSONModule
        FileModule
        DateModule
        UUID
        Random
        HtmlModule
        HttpModule
    }
    Ok(())
}
