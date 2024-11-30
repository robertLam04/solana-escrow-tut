import { serializeInitEscrow } from "./instructions/initEscrow";

function displayBufferContents(expectedYFromAcceptor: string): void {
    
    const bigintExpectedYFromAcceptor = BigInt(expectedYFromAcceptor);
    const data = serializeInitEscrow(bigintExpectedYFromAcceptor);

    // Log the Buffer in different formats
    console.log("Buffer size:", data.length);
    console.log("Raw Buffer:", data);
    console.log("Hexadecimal:", data.toString("hex"));
    console.log("JSON Representation:", data.toJSON());
    console.log("String (utf-8):", data.toString("utf-8"));
}

// Read expectedYFromAcceptor from command-line arguments
const expectedYFromAcceptor = process.argv[2];
if (!expectedYFromAcceptor) {
    console.error("Usage: ts-node display-buffer.ts <expectedYFromAcceptor>");
    process.exit(1);
}

displayBufferContents(expectedYFromAcceptor);
