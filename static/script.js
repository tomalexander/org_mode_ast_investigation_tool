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
    outputElement.innerHTML = "";
    astTreeElement.innerHTML = "";
}

function renderParseResponse(response) {
    clearOutput();
    console.log(response);
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
        wrappedLine.textContent = line ? line : "\n";
        outputElement.appendChild(wrappedLine);
    }
}

function renderAstTree(response) {
    for (let list of response.lists) {
        renderAstList(response.input, 0, list);
    }
}

function renderAstList(originalSource, depth, list) {
    const listElem = document.createElement("div");
    listElem.classList.add("ast_node");

    let sourceForList = originalSource.slice(list.position.start_character - 1, list.position.end_character - 1);
    // Since sourceForList is a string, JSON.stringify will escape with backslashes and wrap the text in quotation marks, ensuring that the string ends up on a single line. Coincidentally, this is the behavior we want.
    let escapedSource = JSON.stringify(sourceForList);

    listElem.innerText = `List: ${escapedSource}`;
    listElem.style.marginLeft = `${depth * 20}px`;
    astTreeElement.appendChild(listElem);
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
    const codeLineElement = document.querySelector(`.${htmlName} > code:nth-child(${childOffset})`);
  codeLineElement?.classList.add("highlighted")
}

function unhighlightLine(htmlName, lineOffset) {
  const childOffset = lineOffset + 1;
    const codeLineElement = document.querySelector(`.${htmlName} > code:nth-child(${childOffset})`);
  codeLineElement?.classList.remove("highlighted")
}
