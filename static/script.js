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
}

inputElement.addEventListener("input", async () => {
    let orgSource = inputElement.value;
    if (inFlightRequest != null) {
        inFlightRequest.abort();
        inFlightRequest = null;
    }

    let newRequest = abortableFetch("/parse", {
        method: "POST",
        cache: "no-cache",
        body: orgSource,
    });
    inFlightRequest = newRequest;

    let response = await inFlightRequest.ready;
    renderParseResponse(await response.json());
});
