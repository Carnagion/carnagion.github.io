CONTENT := "content"
ASSETS := "assets"
TARGET := ".html"

build content=CONTENT assets=ASSETS target=TARGET: (clean target)
    content=$(find {{ quote(content) }} -type f -name "*.typ" | sort | sed 's/.*/"&"/' | paste -sd, ); \
    assets=$(find {{ quote(assets) }} -type f | sort | sed 's/.*/"&"/' | paste -sd, ); \
    typst compile main.typ {{ quote(target) }} \
        --format bundle \
        --input "content=[$content]" \
        --input "assets=[$assets]" \
        --features bundle,html;

clean target=TARGET:
    rm -rf {{ quote(target) }};
