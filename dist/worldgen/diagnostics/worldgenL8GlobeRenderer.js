const DEFAULT_L5_SAMPLE_COUNT = 10_242;
export function buildGpuPositions(positions) {
    return Float32Array.from(positions);
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
    in vec3 aPosition;
    in vec4 aColor;
    uniform float uYaw;
    uniform float uPitch;
    uniform float uZoom;
    uniform float uClipScaleX;
    uniform float uClipScaleY;
    uniform float uPointScale;
    out vec4 vColor;
    void main() {
      float cy = cos(uYaw);
      float sy = sin(uYaw);
      float cp = cos(uPitch);
      float sp = sin(uPitch);
      float x1 = cy * aPosition.x - sy * aPosition.y;
      float y1 = sy * aPosition.x + cy * aPosition.y;
      float rotatedX = cp * x1 + sp * aPosition.z;
      float rotatedZ = -sp * x1 + cp * aPosition.z;
      if (rotatedX < 0.0) {
        gl_Position = vec4(2.0, 2.0, 0.0, 1.0);
        gl_PointSize = 1.0;
      } else {
        gl_Position = vec4(y1 * uClipScaleX, rotatedZ * uClipScaleY, -rotatedX, 1.0);
        gl_PointSize = clamp(uPointScale * max(1.0, uZoom), 1.0, 40.0);
      }
      vColor = aColor;
    }
  `);
    const fragment = compileShader(gl, gl.FRAGMENT_SHADER, `#version 300 es
    precision mediump float;
    in vec4 vColor;
    uniform float uAlpha;
    out vec4 outColor;
    void main() {
      vec2 point = gl_PointCoord * 2.0 - 1.0;
      if (dot(point, point) > 1.0) discard;
      outColor = vec4(vColor.rgb, vColor.a * uAlpha);
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
    colorBuffer;
    uploadedResult = null;
    uploadedColorKey = '';
    constructor(canvas) {
        this.canvas = canvas;
        const gl = canvas.getContext('webgl2', {
            alpha: false,
            antialias: false,
            depth: false,
            stencil: false,
            preserveDrawingBuffer: false,
            powerPreference: 'high-performance',
        });
        this.gl = gl;
        if (!gl) {
            this.program = null;
            this.positionBuffer = null;
            this.colorBuffer = null;
            return;
        }
        this.program = createProgram(gl);
        this.positionBuffer = gl.createBuffer();
        this.colorBuffer = gl.createBuffer();
        if (!this.positionBuffer || !this.colorBuffer)
            throw new Error('Could not allocate WebGL buffers.');
    }
    get available() { return this.gl !== null && this.program !== null; }
    draw(result, colors, colorKey, camera, width, height, alpha = 0.94) {
        const gl = this.gl;
        const program = this.program;
        if (!gl || !program || !this.positionBuffer || !this.colorBuffer)
            return false;
        if (this.canvas.width !== width)
            this.canvas.width = width;
        if (this.canvas.height !== height)
            this.canvas.height = height;
        gl.viewport(0, 0, width, height);
        gl.disable(gl.DEPTH_TEST);
        gl.disable(gl.CULL_FACE);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.clearColor(8 / 255, 16 / 255, 26 / 255, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.useProgram(program);
        if (this.uploadedResult !== result) {
            gl.bindBuffer(gl.ARRAY_BUFFER, this.positionBuffer);
            gl.bufferData(gl.ARRAY_BUFFER, buildGpuPositions(result.positions), gl.STATIC_DRAW);
            this.uploadedResult = result;
            this.uploadedColorKey = '';
        }
        if (this.uploadedColorKey !== colorKey) {
            gl.bindBuffer(gl.ARRAY_BUFFER, this.colorBuffer);
            gl.bufferData(gl.ARRAY_BUFFER, colors, gl.DYNAMIC_DRAW);
            this.uploadedColorKey = colorKey;
        }
        const positionLocation = gl.getAttribLocation(program, 'aPosition');
        gl.bindBuffer(gl.ARRAY_BUFFER, this.positionBuffer);
        gl.enableVertexAttribArray(positionLocation);
        gl.vertexAttribPointer(positionLocation, 3, gl.FLOAT, false, 0, 0);
        const colorLocation = gl.getAttribLocation(program, 'aColor');
        gl.bindBuffer(gl.ARRAY_BUFFER, this.colorBuffer);
        gl.enableVertexAttribArray(colorLocation);
        gl.vertexAttribPointer(colorLocation, 4, gl.UNSIGNED_BYTE, true, 0, 0);
        const radius = Math.min(width, height) * 0.44 * camera.zoom;
        gl.uniform1f(gl.getUniformLocation(program, 'uYaw'), camera.yaw);
        gl.uniform1f(gl.getUniformLocation(program, 'uPitch'), camera.pitch);
        gl.uniform1f(gl.getUniformLocation(program, 'uZoom'), camera.zoom);
        gl.uniform1f(gl.getUniformLocation(program, 'uClipScaleX'), radius * 2 / width);
        gl.uniform1f(gl.getUniformLocation(program, 'uClipScaleY'), radius * 2 / height);
        gl.uniform1f(gl.getUniformLocation(program, 'uPointScale'), 1.45);
        gl.uniform1f(gl.getUniformLocation(program, 'uAlpha'), alpha);
        gl.drawArrays(gl.POINTS, 0, result.metrics.fineSampleCount);
        return true;
    }
    dispose() {
        const gl = this.gl;
        if (!gl)
            return;
        if (this.positionBuffer)
            gl.deleteBuffer(this.positionBuffer);
        if (this.colorBuffer)
            gl.deleteBuffer(this.colorBuffer);
        if (this.program)
            gl.deleteProgram(this.program);
        this.uploadedResult = null;
        this.uploadedColorKey = '';
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
