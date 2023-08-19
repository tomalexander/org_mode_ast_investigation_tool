let inFlightRequest = null;
const inputElement = document.querySelector("#org-input");
const outputElement = document.querySelector("#parse-output");
const astTreeElement = document.querySelector("#ast-tree");

function abortableFetch(request, options) {
    const controller = new AbortController();
    const signal = controller.signal;

    return {
        abort: () => controller.abort(),
        ready: fetch(request, { ...options, signal })
    };
}

function clearOutput() {
    clearActiveAstNode();
    outputElement.innerHTML = "";
    astTreeElement.innerHTML = "";
}

function renderParseResponse(response) {
    clearOutput();
    renderSourceBox(response);
    renderAstTree(response);
}

function renderSourceBox(response) {
    const lines = response.input.split(/\r?\n/);
    const numLines = lines.length;
    const numDigits = Math.log10(numLines) + 1;

    outputElement.style.paddingLeft = `calc(${numDigits + 1}ch + 10px)`;

    for (let line of lines) {
        let wrappedLine = document.createElement("code");
        if (line !== "" && line !== null) {
            for (let chr of line) {
                // Please forgive me
                let wrappedCharacter = document.createElement("span");
                wrappedCharacter.textContent = chr;
                wrappedLine.appendChild(wrappedCharacter);
            }
        } else {
            let wrappedCharacter = document.createElement("span");
            wrappedCharacter.textContent = "\n";
            wrappedLine.appendChild(wrappedCharacter);
        }
        outputElement.appendChild(wrappedLine);
    }
}

function renderAstTree(response) {
    renderAstNode(response.input, 0, response.tree);
}

function renderAstNode(originalSource, depth, astNode) {
    const nodeElem = document.createElement("div");
    nodeElem.classList.add("ast_node");

    let sourceForNode = unicodeAwareSlice(originalSource, astNode.position.start_character - 1, astNode.position.end_character - 1);
    // Since sourceForList is a string, JSON.stringify will escape with backslashes and wrap the text in quotation marks, ensuring that the string ends up on a single line. Coincidentally, this is the behavior we want.
    let escapedSource = JSON.stringify(sourceForNode);

    nodeElem.innerText = `${astNode.name}: ${escapedSource}`;
    nodeElem.style.marginLeft = `${depth * 20}px`;
    nodeElem.dataset.startLine = astNode.position.start_line;
    nodeElem.dataset.endLine = astNode.position.end_line;
    nodeElem.dataset.startCharacter = astNode.position.start_character;
    nodeElem.dataset.endCharacter = astNode.position.end_character;

    nodeElem.addEventListener("click", () => {
        setActiveAstNode(nodeElem, originalSource);
    });

    astTreeElement.appendChild(nodeElem);
    for (let child of astNode.children) {
        renderAstNode(originalSource, depth + 1, child);
    }
}

function clearActiveAstNode() {
    for (let elem of document.querySelectorAll("#ast-tree .ast_node.highlighted")) {
        elem.classList.remove("highlighted");
    }
    for (let elem of document.querySelectorAll("#parse-output > code.highlighted")) {
        elem.classList.remove("highlighted");
    }
    for (let elem of document.querySelectorAll("#parse-output > code > span")) {
        elem.classList.remove("highlighted");
    }
}

function setActiveAstNode(elem, originalSource) {
    clearActiveAstNode();
    elem.classList.add("highlighted");
    let startLine = parseInt(elem.dataset.startLine, 10);
    let endLine = parseInt(elem.dataset.endLine, 10);
    let startCharacter = parseInt(elem.dataset.startCharacter, 10);
    let endCharacter = parseInt(elem.dataset.endCharacter, 10);
    for (let line = startLine; line < endLine; ++line) {
        highlightLine("parse-output", line - 1);
    }
    highlightCharacters("parse-output", originalSource, startCharacter, endCharacter);
}

inputElement.addEventListener("input", async () => {
    let orgSource = inputElement.value;
    if (inFlightRequest != null) {
        inFlightRequest.abort();
        inFlightRequest = null;
    }
    clearOutput();

    let newRequest = abortableFetch("/parse", {
        method: "POST",
        cache: "no-cache",
        body: orgSource,
    });
    inFlightRequest = newRequest;

    let response = null;
    try {
        response = await inFlightRequest.ready;
    }
    catch (err) {
        if (err.name === "AbortError") return;
    }
    renderParseResponse(await response.json());
});

function highlightLine(htmlName, lineOffset) {
  const childOffset = lineOffset + 1;
    const codeLineElement = document.querySelector(`#${htmlName} > code:nth-child(${childOffset})`);
  codeLineElement?.classList.add("highlighted")
}

function highlightCharacters(htmlName, originalSource, startCharacter, endCharacter) {
    let sourceBefore = unicodeAwareSlice(originalSource, 0, startCharacter - 1);
    let precedingLineBreak = unicodeAwareLastIndexOfCharacter(sourceBefore, "\n");
    let characterIndexOnLine = precedingLineBreak !== -1 ? startCharacter - precedingLineBreak - 1 : startCharacter;
    let lineNumber = (sourceBefore.match(/\r?\n/g) || '').length + 1;

    for (let characterIndex = startCharacter; characterIndex < endCharacter; ++characterIndex) {
        document.querySelector(`#${htmlName} > code:nth-child(${lineNumber}) > span:nth-child(${characterIndexOnLine})`)?.classList.add("highlighted");
        if (unicodeAwareCharAtOffset(originalSource, characterIndex - 1) == "\n") {
            ++lineNumber;
            characterIndexOnLine = 1;
        } else {
            ++characterIndexOnLine;
        }
    }

}

function unicodeAwareSlice(text, start, end) {
    // Boooo javascript
    let i = 0;
    let output = "";
    for (chr of text) {
        if (i >= end) {
            break;
        }
        if (i >= start) {
            output += chr;
        }
        ++i;
    }
    return output;
}

function unicodeAwareLastIndexOfCharacter(haystack, needle) {
    // Boooo javascript
    let i = 0;
    let found = -1;
    for (chr of haystack) {
        if (chr == needle) {
            found = i;
        }
        ++i;
    }
    return found;
}

function unicodeAwareCharAtOffset(text, offset) {
    // Boooo javascript
    let i = offset;
    for (chr of text) {
        if (i == 0) {
            return chr;
        }
        --i;
    }
}
