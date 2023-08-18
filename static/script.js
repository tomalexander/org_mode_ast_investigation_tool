let inFlightRequest = null;
const inputElement = document.querySelector("#org-input");
const outputElement = document.querySelector("#parse-output");

function abortableFetch(request, options) {
    const controller = new AbortController();
    const signal = controller.signal;

    return {
        abort: () => controller.abort(),
        ready: fetch(request, { ...options, signal })
    };
}

async function renderParseResponse(response) {
    console.log(response);
    outputElement.innerHTML = "";
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

inputElement.addEventListener("input", async () => {
    let orgSource = inputElement.value;
    if (inFlightRequest != null) {
        inFlightRequest.abort();
        inFlightRequest = null;
    }
    outputElement.innerHTML = "";

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
