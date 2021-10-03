import * as bip from "batch-image-processor"
import { saveAs } from "file-saver"

function setup() {
    const fileInput = document.getElementById('zip-upload')
    fileInput.addEventListener('change', function (event)  {
        load(event)
    })
}

async function load(event) {
    const file = event.target.files[0]

    if (!file) {
        return
    }

    const buffer = await readFile(file)
    const zipReader = await bip.ArchiveReader.new(buffer)

    await progressBar(zipReader)

    const filenames = await zipReader.extractFilenames()

    const zipWriter = await bip.ArchiveWriter.new();
    await zipWriter.transformImages(zipReader)

    const $template = document.querySelector('[data-js=list-item-template]')
    const $list = document.querySelector('[data-js=list]')

    for (const filename of filenames) {
        const $item = document.importNode($template.content, true)
        const $filename = $item.querySelector('[data-js="filename"]')
        const $extract = $item.querySelector('[data-js="extract"]')

        $filename.textContent = filename
        $extract.addEventListener('click', async () => {
            await extractFile(zipReader, filename)
        })

        $list.appendChild($item)
    }

    const $zipDownloadButton = document.getElementById("zip-download")
    $zipDownloadButton.addEventListener('click', async () => {
        await extractZip(zipWriter)
    })
    $zipDownloadButton.style.display = "block"
}

async function extractZip(zipWriter) {
    const writerBuffer = zipWriter.extractBinary()
    const blob = new Blob([writerBuffer], {type: 'application/octet-stream'})
    saveAs(blob, 'images.zip')
}

async function extractFile(zipReader, filename) {
    const buffer = await zipReader.extractBinary(filename)
    const blob = new Blob([buffer], {type: 'application/octet-stream'})
    const basename = filename.split('/').pop()
    saveAs(blob, basename)
}

async function progressBar(zipReader) {
    const bar = document.getElementById('js-progressbar');

    const max = parseInt(await zipReader.getLength());
    let unit = 100
    if (max > 0) {
        unit = 100 / max
    }

    const animate = setInterval(async function () {
        const idx = parseInt(await zipReader.getProcessIdx())
        bar.value += idx * unit;

        if (bar.value >= bar.max) {
            clearInterval(animate);
        }
    }, 100);
}

async function readFile(fileInput) {
    const buffer = await new Promise((resolve, reject) => {
        const reader = new FileReader()

        reader.addEventListener('loadend', () => resolve(reader.result))
        reader.addEventListener('error', () => reject)

        reader.readAsArrayBuffer(fileInput)
    })

    return new Uint8Array(buffer)
}

if (document.readState !== 'loading') {
    setup()
} else {
    window.addEventListener('DOMContentLoaded', setup);
}
