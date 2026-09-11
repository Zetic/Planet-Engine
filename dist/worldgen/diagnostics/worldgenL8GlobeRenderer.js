const DEFAULT_L5_SAMPLE_COUNT = 10_242;
export function buildGpuPositions(positions) {
    return Float32Array.from(positions);
}
function readVec3(positions, sample) {
    const offset = sample * 3;
    return [positions[offset], positions[offset + 1], positions[offset + 2]];
}
function vecSub(a, b) { return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]; }
function vecDot(a, b) { return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]; }
function vecCross(a, b) {
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}
function vecNormalize(value) {
    const magnitude = Math.hypot(value[0], value[1], value[2]);
    if (!(magnitude > 1e-12))
        throw new Error('Degenerate L8 dual-cell geometry.');
    return [value[0] / magnitude, value[1] / magnitude, value[2] / magnitude];
}
function tangentBasis(up) {
    const reference = Math.abs(up[2]) < 0.9 ? [0, 0, 1] : [0, 1, 0];
    const east = vecNormalize(vecCross(reference, up));
    return [east, vecCross(up, east)];
}
function sphericalCircumcenter(a, b, c) {
    let center = vecNormalize(vecCross(vecSub(b, a), vecSub(c, a)));
    if (vecDot(center, a) < 0)
        center = [-center[0], -center[1], -center[2]];
    return center;
}
export function buildDualCellGpuMesh(result) {
    const sampleCount = result.metrics.fineSampleCount;
    if (result.neighborOffsets.length !== sampleCount + 1)
        throw new Error('Invalid L8 neighbor offsets.');
    const totalDegree = result.neighborOffsets[sampleCount];
    if (totalDegree !== result.neighbors.length)
        throw new Error('Invalid L8 neighbor adjacency length.');
    let maxDegree = 0;
    for (let sample = 0; sample < sampleCount; sample += 1) {
        maxDegree = Math.max(maxDegree, result.neighborOffsets[sample + 1] - result.neighborOffsets[sample]);
    }
    if (maxDegree < 3)
        throw new Error('L8 topology has no valid dual cells.');
    const vertexCount = sampleCount + totalDegree;
    const positions = new Float32Array(vertexCount * 3);
    const cellIds = new Uint32Array(vertexCount);
    const indices = new Uint32Array(totalDegree * 3);
    const boundaryIndices = new Uint32Array(totalDegree * 2);
    const orderedSamples = new Uint32Array(maxDegree);
    const orderedAngles = new Float64Array(maxDegree);
    const cornerIds = new Uint32Array(maxDegree);
    let vertexCursor = 0;
    let indexCursor = 0;
    let boundaryCursor = 0;
    const writeVertex = (vertex, cellId) => {
        const vertexId = vertexCursor++;
        const offset = vertexId * 3;
        positions[offset] = vertex[0];
        positions[offset + 1] = vertex[1];
        positions[offset + 2] = vertex[2];
        cellIds[vertexId] = cellId;
        return vertexId;
    };
    for (let sample = 0; sample < sampleCount; sample += 1) {
        const start = result.neighborOffsets[sample];
        const end = result.neighborOffsets[sample + 1];
        const degree = end - start;
        if (degree < 3)
            throw new Error(`L8 cell ${sample} has invalid degree ${degree}.`);
        const center = readVec3(result.positions, sample);
        const [east, north] = tangentBasis(center);
        // Degree is only five or six on canonical geodesic topologies. Reuse fixed scratch
        // buffers and insertion-sort the tiny ring to avoid millions of transient objects
        // while constructing the 655k-cell L8 GPU mesh.
        for (let local = 0; local < degree; local += 1) {
            const neighbor = result.neighbors[start + local];
            const direction = readVec3(result.positions, neighbor);
            const angle = Math.atan2(vecDot(direction, north), vecDot(direction, east));
            let slot = local;
            while (slot > 0 && orderedAngles[slot - 1] > angle) {
                orderedAngles[slot] = orderedAngles[slot - 1];
                orderedSamples[slot] = orderedSamples[slot - 1];
                slot -= 1;
            }
            orderedAngles[slot] = angle;
            orderedSamples[slot] = neighbor;
        }
        const base = writeVertex(center, sample);
        for (let index = 0; index < degree; index += 1) {
            const a = readVec3(result.positions, orderedSamples[index]);
            const b = readVec3(result.positions, orderedSamples[(index + 1) % degree]);
            cornerIds[index] = writeVertex(sphericalCircumcenter(center, a, b), sample);
        }
        for (let index = 0; index < degree; index += 1) {
            indices[indexCursor++] = base;
            indices[indexCursor++] = cornerIds[index];
            indices[indexCursor++] = cornerIds[(index + 1) % degree];
            boundaryIndices[boundaryCursor++] = cornerIds[index];
            boundaryIndices[boundaryCursor++] = cornerIds[(index + 1) % degree];
        }
    }
    if (vertexCursor !== vertexCount || indexCursor !== indices.length || boundaryCursor !== boundaryIndices.length) {
        throw new Error(`L8 dual-cell mesh size mismatch: ${vertexCursor}/${vertexCount} vertices, ${indexCursor}/${indices.length} triangle indices, ${boundaryCursor}/${boundaryIndices.length} boundary indices.`);
    }
    return {
        positions,
        cellIds,
        indices,
        boundaryIndices,
        vertexCount,
        triangleCount: indices.length / 3,
        boundaryIndexCount: boundaryIndices.length,
    };
}
export function collectViewportSampleIndices(x, y, visible, width, height, marginPixels = 24) {
    if (x.length !== y.length || x.length !== visible.length) {
        throw new Error('Projected L8 sample buffers must have matching lengths.');
    }
    const margin = Math.max(0, marginPixels);
    const samples = [];
    for (let sample = 0; sample < x.length; sample += 1) {
        if (!visible[sample])
            continue;
        const px = x[sample];
        const py = y[sample];
        if (px < -margin || px > width + margin || py < -margin || py > height + margin)
            continue;
        samples.push(sample);
    }
    return Uint32Array.from(samples);
}
function compileShader(gl, type, source) {
    const shader = gl.createShader(type);
    if (!shader)
        throw new Error('Could not allocate WebGL shader.');
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        const message = gl.getShaderInfoLog(shader) ?? 'unknown shader compile error';
        gl.deleteShader(shader);
        throw new Error(message);
    }
    return shader;
}
function createProgram(gl) {
    const vertex = compileShader(gl, gl.VERTEX_SHADER, `#version 300 es
    precision highp float;
    precision highp int;
    in vec3 aPosition;
    in uint aCellId;
    uniform float uYaw;
    uniform float uPitch;
    uniform float uClipScaleX;
    uniform float uClipScaleY;
    flat out uint vCellId;
    out float vFront;
    void main() {
      float cy = cos(uYaw);
      float sy = sin(uYaw);
      float cp = cos(uPitch);
      float sp = sin(uPitch);
      float x1 = cy * aPosition.x - sy * aPosition.y;
      float y1 = sy * aPosition.x + cy * aPosition.y;
      float rotatedX = cp * x1 + sp * aPosition.z;
      float rotatedZ = -sp * x1 + cp * aPosition.z;
      // Keep the front pole comfortably inside clip space. The previous -rotatedX
      // mapping put valid unit-sphere geometry directly on z=-1 and could expose a
      // camera-centered clear-color cap through precision/clipping artifacts.
      gl_Position = vec4(y1 * uClipScaleX, rotatedZ * uClipScaleY, -rotatedX * 0.5, 1.0);
      vCellId = aCellId;
      vFront = rotatedX;
    }
  `);
    const fragment = compileShader(gl, gl.FRAGMENT_SHADER, `#version 300 es
    precision highp float;
    precision highp int;
    flat in uint vCellId;
    in float vFront;
    uniform sampler2D uColorTexture;
    uniform int uColorTextureWidth;
    uniform float uAlpha;
    uniform int uUseSolidColor;
    uniform vec4 uSolidColor;
    out vec4 outColor;
    void main() {
      if (vFront < 0.0) discard;
      if (uUseSolidColor != 0) {
        outColor = uSolidColor;
        return;
      }
      int cell = int(vCellId);
      ivec2 texel = ivec2(cell % uColorTextureWidth, cell / uColorTextureWidth);
      vec4 color = texelFetch(uColorTexture, texel, 0);
      outColor = vec4(color.rgb, color.a * uAlpha);
    }
  `);
    const program = gl.createProgram();
    if (!program)
        throw new Error('Could not allocate WebGL program.');
    gl.attachShader(program, vertex);
    gl.attachShader(program, fragment);
    gl.linkProgram(program);
    gl.deleteShader(vertex);
    gl.deleteShader(fragment);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        const message = gl.getProgramInfoLog(program) ?? 'unknown program link error';
        gl.deleteProgram(program);
        throw new Error(message);
    }
    return program;
}
export class L8GlobeRenderer {
    canvas;
    gl;
    program;
    positionBuffer;
    cellIdBuffer;
    indexBuffer;
    boundaryIndexBuffer;
    colorTexture;
    uploadedResult = null;
    uploadedColorKey = '';
    uploadedIndexCount = 0;
    uploadedBoundaryIndexCount = 0;
    colorTextureWidth = 1024;
    colorTextureScratch = new Uint8Array(0);
    constructor(canvas) {
        this.canvas = canvas;
        const gl = canvas.getContext('webgl2', {
            alpha: false,
            antialias: false,
            depth: true,
            stencil: false,
            preserveDrawingBuffer: false,
            powerPreference: 'high-performance',
        });
        this.gl = gl;
        if (!gl) {
            this.program = null;
            this.positionBuffer = null;
            this.cellIdBuffer = null;
            this.indexBuffer = null;
            this.boundaryIndexBuffer = null;
            this.colorTexture = null;
            return;
        }
        this.program = createProgram(gl);
        this.positionBuffer = gl.createBuffer();
        this.cellIdBuffer = gl.createBuffer();
        this.indexBuffer = gl.createBuffer();
        this.boundaryIndexBuffer = gl.createBuffer();
        this.colorTexture = gl.createTexture();
        if (!this.positionBuffer || !this.cellIdBuffer || !this.indexBuffer || !this.boundaryIndexBuffer || !this.colorTexture) {
            throw new Error('Could not allocate WebGL dual-cell resources.');
        }
    }
    get available() { return this.gl !== null && this.program !== null; }
    uploadGeometry(result) {
        const gl = this.gl;
        const mesh = buildDualCellGpuMesh(result);
        gl.bindBuffer(gl.ARRAY_BUFFER, this.positionBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, mesh.positions, gl.STATIC_DRAW);
        gl.bindBuffer(gl.ARRAY_BUFFER, this.cellIdBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, mesh.cellIds, gl.STATIC_DRAW);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, mesh.indices, gl.STATIC_DRAW);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.boundaryIndexBuffer);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, mesh.boundaryIndices, gl.STATIC_DRAW);
        this.uploadedIndexCount = mesh.indices.length;
        this.uploadedBoundaryIndexCount = mesh.boundaryIndices.length;
        this.uploadedResult = result;
        this.uploadedColorKey = '';
    }
    uploadColors(colors, sampleCount) {
        const gl = this.gl;
        const maxTextureSize = Number(gl.getParameter(gl.MAX_TEXTURE_SIZE));
        this.colorTextureWidth = Math.min(1024, maxTextureSize);
        const height = Math.ceil(sampleCount / this.colorTextureWidth);
        if (height > maxTextureSize)
            throw new Error('L8 color texture exceeds WebGL2 texture limits.');
        const required = this.colorTextureWidth * height * 4;
        if (this.colorTextureScratch.length !== required)
            this.colorTextureScratch = new Uint8Array(required);
        this.colorTextureScratch.set(colors);
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, this.colorTexture);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, this.colorTextureWidth, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, this.colorTextureScratch);
    }
    draw(result, colors, colorKey, camera, width, height, alpha = 0.94, showCellBorders = false, selectedCell = null) {
        const gl = this.gl;
        const program = this.program;
        if (!gl || !program || !this.positionBuffer || !this.cellIdBuffer || !this.indexBuffer || !this.boundaryIndexBuffer || !this.colorTexture)
            return false;
        if (this.canvas.width !== width)
            this.canvas.width = width;
        if (this.canvas.height !== height)
            this.canvas.height = height;
        gl.viewport(0, 0, width, height);
        gl.enable(gl.DEPTH_TEST);
        gl.depthFunc(gl.LEQUAL);
        gl.disable(gl.CULL_FACE);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.clearColor(8 / 255, 16 / 255, 26 / 255, 1);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        gl.useProgram(program);
        if (this.uploadedResult !== result)
            this.uploadGeometry(result);
        if (this.uploadedColorKey !== colorKey) {
            this.uploadColors(colors, result.metrics.fineSampleCount);
            this.uploadedColorKey = colorKey;
        }
        const positionLocation = gl.getAttribLocation(program, 'aPosition');
        gl.bindBuffer(gl.ARRAY_BUFFER, this.positionBuffer);
        gl.enableVertexAttribArray(positionLocation);
        gl.vertexAttribPointer(positionLocation, 3, gl.FLOAT, false, 0, 0);
        const cellIdLocation = gl.getAttribLocation(program, 'aCellId');
        gl.bindBuffer(gl.ARRAY_BUFFER, this.cellIdBuffer);
        gl.enableVertexAttribArray(cellIdLocation);
        gl.vertexAttribIPointer(cellIdLocation, 1, gl.UNSIGNED_INT, 0, 0);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, this.colorTexture);
        const radius = Math.min(width, height) * 0.44 * camera.zoom;
        gl.uniform1f(gl.getUniformLocation(program, 'uYaw'), camera.yaw);
        gl.uniform1f(gl.getUniformLocation(program, 'uPitch'), camera.pitch);
        gl.uniform1f(gl.getUniformLocation(program, 'uClipScaleX'), radius * 2 / width);
        gl.uniform1f(gl.getUniformLocation(program, 'uClipScaleY'), radius * 2 / height);
        gl.uniform1i(gl.getUniformLocation(program, 'uColorTexture'), 0);
        gl.uniform1i(gl.getUniformLocation(program, 'uColorTextureWidth'), this.colorTextureWidth);
        gl.uniform1f(gl.getUniformLocation(program, 'uAlpha'), alpha);
        gl.uniform1i(gl.getUniformLocation(program, 'uUseSolidColor'), 0);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
        gl.drawElements(gl.TRIANGLES, this.uploadedIndexCount, gl.UNSIGNED_INT, 0);
        const borderFade = showCellBorders ? Math.max(0, Math.min(1, (camera.zoom - 2.0) / 2.5)) : 0;
        if (borderFade > 0 || selectedCell !== null) {
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.boundaryIndexBuffer);
            gl.uniform1i(gl.getUniformLocation(program, 'uUseSolidColor'), 1);
            gl.lineWidth(1);
            if (borderFade > 0) {
                gl.uniform4f(gl.getUniformLocation(program, 'uSolidColor'), 225 / 255, 236 / 255, 246 / 255, 0.62 * borderFade);
                gl.drawElements(gl.LINES, this.uploadedBoundaryIndexCount, gl.UNSIGNED_INT, 0);
            }
            if (selectedCell !== null && selectedCell >= 0 && selectedCell < result.metrics.fineSampleCount) {
                const start = result.neighborOffsets[selectedCell];
                const end = result.neighborOffsets[selectedCell + 1];
                const degree = end - start;
                gl.uniform4f(gl.getUniformLocation(program, 'uSolidColor'), 93 / 255, 224 / 255, 1, 1);
                gl.drawElements(gl.LINES, degree * 2, gl.UNSIGNED_INT, start * 2 * Uint32Array.BYTES_PER_ELEMENT);
            }
            gl.uniform1i(gl.getUniformLocation(program, 'uUseSolidColor'), 0);
        }
        return true;
    }
    dispose() {
        const gl = this.gl;
        if (!gl)
            return;
        if (this.positionBuffer)
            gl.deleteBuffer(this.positionBuffer);
        if (this.cellIdBuffer)
            gl.deleteBuffer(this.cellIdBuffer);
        if (this.indexBuffer)
            gl.deleteBuffer(this.indexBuffer);
        if (this.boundaryIndexBuffer)
            gl.deleteBuffer(this.boundaryIndexBuffer);
        if (this.colorTexture)
            gl.deleteTexture(this.colorTexture);
        if (this.program)
            gl.deleteProgram(this.program);
        this.uploadedResult = null;
        this.uploadedColorKey = '';
        this.uploadedIndexCount = 0;
        this.uploadedBoundaryIndexCount = 0;
        this.colorTextureScratch = new Uint8Array(0);
    }
}
function hslToRgb(hueDegrees, saturationPercent, lightnessPercent) {
    const hue = ((hueDegrees % 360) + 360) % 360 / 360;
    const saturation = Math.max(0, Math.min(1, saturationPercent / 100));
    const lightness = Math.max(0, Math.min(1, lightnessPercent / 100));
    if (saturation === 0) {
        const gray = Math.round(lightness * 255);
        return [gray, gray, gray];
    }
    const q = lightness < 0.5 ? lightness * (1 + saturation) : lightness + saturation - lightness * saturation;
    const p = 2 * lightness - q;
    const channel = (offset) => {
        let t = hue + offset;
        if (t < 0)
            t += 1;
        if (t > 1)
            t -= 1;
        if (t < 1 / 6)
            return p + (q - p) * 6 * t;
        if (t < 1 / 2)
            return q;
        if (t < 2 / 3)
            return p + (q - p) * (2 / 3 - t) * 6;
        return p;
    };
    return [Math.round(channel(1 / 3) * 255), Math.round(channel(0) * 255), Math.round(channel(-1 / 3) * 255)];
}
function cssColorToRgba(color) {
    if (/^#[0-9a-fA-F]{6}$/.test(color)) {
        return [parseInt(color.slice(1, 3), 16), parseInt(color.slice(3, 5), 16), parseInt(color.slice(5, 7), 16), 255];
    }
    const hsl = color.match(/^hsl\(\s*(-?[0-9.]+)\s+([0-9.]+)%\s+([0-9.]+)%\s*\)$/i);
    if (hsl) {
        const [red, green, blue] = hslToRgb(Number(hsl[1]), Number(hsl[2]), Number(hsl[3]));
        return [red, green, blue, 255];
    }
    throw new Error(`Unsupported GPU sample color: ${color}`);
}
export function buildRgbaColors(sampleCount, sampler) {
    const output = new Uint8Array(sampleCount * 4);
    const cache = new Map();
    for (let sample = 0; sample < sampleCount; sample += 1) {
        const css = sampler(sample);
        let rgba = cache.get(css);
        if (!rgba) {
            rgba = cssColorToRgba(css);
            cache.set(css, rgba);
        }
        const offset = sample * 4;
        output[offset] = rgba[0];
        output[offset + 1] = rgba[1];
        output[offset + 2] = rgba[2];
        output[offset + 3] = rgba[3];
    }
    return output;
}
export function screenToWorldDirection(canvasX, canvasY, width, height, camera) {
    const radius = Math.min(width, height) * 0.44 * camera.zoom;
    const y1 = (canvasX - width / 2) / radius;
    const rotatedZ = -(canvasY - height / 2) / radius;
    const radialSquared = y1 * y1 + rotatedZ * rotatedZ;
    if (radialSquared > 1)
        return null;
    const rotatedX = Math.sqrt(Math.max(0, 1 - radialSquared));
    const cp = Math.cos(camera.pitch);
    const sp = Math.sin(camera.pitch);
    const x1 = cp * rotatedX - sp * rotatedZ;
    const z = sp * rotatedX + cp * rotatedZ;
    const cy = Math.cos(camera.yaw);
    const sy = Math.sin(camera.yaw);
    const x = cy * x1 + sy * y1;
    const y = -sy * x1 + cy * y1;
    return [x, y, z];
}
export function cameraForWorldDirectionAtScreen(direction, canvasX, canvasY, width, height, zoom, current) {
    const radius = Math.min(width, height) * 0.44 * zoom;
    const targetHorizontal = (canvasX - width / 2) / radius;
    const targetVertical = -(canvasY - height / 2) / radius;
    if (targetHorizontal * targetHorizontal + targetVertical * targetVertical >= 0.999999) {
        return { yaw: current.yaw, pitch: current.pitch, zoom };
    }
    let yaw = current.yaw;
    for (let iteration = 0; iteration < 6; iteration += 1) {
        const cy = Math.cos(yaw), sy = Math.sin(yaw);
        const x1 = cy * direction[0] - sy * direction[1];
        const y1 = sy * direction[0] + cy * direction[1];
        const error = y1 - targetHorizontal;
        if (Math.abs(error) < 1e-12 || Math.abs(x1) < 1e-8)
            break;
        yaw -= error / x1;
    }
    const cy = Math.cos(yaw), sy = Math.sin(yaw);
    const x1 = cy * direction[0] - sy * direction[1];
    let pitch = current.pitch;
    for (let iteration = 0; iteration < 6; iteration += 1) {
        const cp = Math.cos(pitch), sp = Math.sin(pitch);
        const rotatedX = cp * x1 + sp * direction[2];
        const rotatedZ = -sp * x1 + cp * direction[2];
        const error = rotatedZ - targetVertical;
        if (Math.abs(error) < 1e-12 || Math.abs(rotatedX) < 1e-8)
            break;
        pitch += error / rotatedX;
    }
    pitch = Math.max(-1.45, Math.min(1.45, pitch));
    return { yaw, pitch, zoom };
}
function sampleDot(result, sample, direction) {
    const offset = sample * 3;
    return result.positions[offset] * direction[0]
        + result.positions[offset + 1] * direction[1]
        + result.positions[offset + 2] * direction[2];
}
export function pickNearestSample(result, direction, coarseSeedCount = DEFAULT_L5_SAMPLE_COUNT) {
    const count = result.metrics.fineSampleCount;
    const seedCount = Math.min(count, coarseSeedCount);
    let bestSample = 0;
    let bestDot = Number.NEGATIVE_INFINITY;
    for (let sample = 0; sample < seedCount; sample += 1) {
        const dot = sampleDot(result, sample, direction);
        if (dot > bestDot) {
            bestDot = dot;
            bestSample = sample;
        }
    }
    while (true) {
        let improved = false;
        const start = result.neighborOffsets[bestSample];
        const end = result.neighborOffsets[bestSample + 1];
        for (let cursor = start; cursor < end; cursor += 1) {
            const neighbor = result.neighbors[cursor];
            const dot = sampleDot(result, neighbor, direction);
            if (dot > bestDot + 1e-12) {
                bestDot = dot;
                bestSample = neighbor;
                improved = true;
            }
        }
        if (!improved)
            return bestSample;
    }
}
