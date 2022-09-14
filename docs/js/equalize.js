for (let target of document.getElementsByClassName("equalize")) {
    equalize(target);
}

function equalize(target) {
    let children = Array.from(target.children);
    for (let child of children) {
        target.removeChild(child);
    }

    // Put all elements into a single row
    target.appendChild(document.createElement("div"));
    for (let child of children) {
        target.firstElementChild.appendChild(child);
    }

    // Sort elements in-place
    while (true) {
        let rowCount = target.childElementCount;
        let colCount = Math.max(...[...target.children].map(child => child.childElementCount));

        if (rowCount === colCount) {
            break;
        }
        if (rowCount < colCount) {
            let max = getMaxColChild(target, false);
            let shifted = max.lastElementChild;
            if (max.nextElementSibling === null) {
                target.append(document.createElement("div"));
            }
            max.removeChild(shifted);
            max.nextElementSibling.prepend(shifted);
        }
        else if (rowCount > colCount) {
            let max = getMaxColChild(target, true);
            let shifted = max.firstElementChild;
            if (max.previousElementSibling === null) {
                target.prepend(document.createElement("div"));
            }
            max.removeChild(shifted);
            max.previousElementSibling.append(shifted);
        }
    }

    // Add flex classes
    target.classList.remove("equalize", "row");
    target.classList.add("flex", "col");
    for (let child of target.children) {
        child.classList.add("gap-tiny", "flex", "row", "wrap");
    }

    return target;
}

function getMaxColChild(element, reverse) {
    let children = [...element.children];
    if (reverse) {
        children.reverse();
    }
    let max = children[0];
    for (let child of children) {
        if (child.childElementCount > max.childElementCount) {
            max = child;
        }
    }
    return max;
}