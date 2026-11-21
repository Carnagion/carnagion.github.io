#let slug(text) = lower(text).replace(regex("[^\\w\\d]+"), "-")

#let components(path) = path.split(regex("[/\\\\]"))
#let filestem(path) = components(path).last().trim(regex("\\.[^\\.]+$"), at: end)

// NOTE: Tinymist LSP defaults to paged export and complains when editing this file if its target isn't overridden to "bundle".
//       However, doing so will cause it to complain when editing any of the other files in the project, since they all include
//       HTML elements. Since we can't tell Tinymist to target both HTML and bundles, we just set it to use HTML export and add
//       this condition here to get it to shut up.
#context if target() == "bundle" {
    let content = json(bytes(sys.inputs.content))
    for (i, src) in content.enumerate() {
        let dst = components(src).slice(1).join("/").replace(regex("\\.typ$"), ".html")
        let id = str(i + 1)
        [#document(dst, include src) #label("content:" + id)]
    }

    // Include all asset files as-is
    let assets = json(bytes(sys.inputs.assets))
    for (i, src) in assets.enumerate() {
        let dst = src // NOTE: No need to strip the base since the assets directory is meant to be bundled as-is unlike content
        let id = str(i + 1)
        [#asset(dst, read(src, encoding: none)) #label("asset:" + id)]
    }
}
