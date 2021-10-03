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

    const bar = document.getElementById('js-progressbar');

    const max = parseInt(zipReader.getLength());
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

    const filenames = await zipReader.extractFilenames()

    const zipWriter = await bip.ArchiveWriter.new();
    await zipWriter.transformImages(zipReader)
    const writerBuffer = zipWriter.extractBinary()
    const blob = new Blob([writerBuffer], {type: 'application/octet-stream'})
    saveAs(blob, 'test.zip')

    // for (const filename of filenames) {
    //     const buffer = await zipReader.extractBinary(filename)
    //     const blob = new Blob([buffer], {type: 'application/octet-stream'})
    //     const basename = filename.split('/').pop()
    //     saveAs(blob, basename)
    // }
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
