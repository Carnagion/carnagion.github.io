use jabberwock::Generator;
use jabberwock::builtin::*;

fn main() {
    println!("Generating...");
    println!("{}", generator().map_or_else(|error| error, |_| String::from("Finished generating.")));
}

fn generator() -> Result<(), String> {
    let mut copier = AssetCopier::new();
    copier.copy("img/", "img/")
        .copy("css/", "css/")
        .copy("js/", "js/");

    let mut generator = Generator::source("in/")?;
    generator.apply(copier)?
        .apply(MarkdownTranspiler::source("md/"))?
        .apply(TemplateTranspiler::source("tmpl/"))?
        .apply(TomlTranspiler::source("vars/"))?
        .apply(HatterTranspiler::new())?;
    generator.destination("out/")
}