export function reconstructAnnualHarmonicFromBasis(mean, cosine, sine, phaseCosine, phaseSine) {
    return mean + cosine * phaseCosine + sine * phaseSine;
}
export function reconstructAnnualHarmonic(mean, cosine, sine, phase) {
    const angle = phase * Math.PI * 2;
    return reconstructAnnualHarmonicFromBasis(mean, cosine, sine, Math.cos(angle), Math.sin(angle));
}
export function mapVectorDelta(eastValue, northValue, latitudeRad, width, height) {
    const speed = Math.hypot(eastValue, northValue);
    if (speed < 1e-9)
        return [0, 0];
    const cosLat = Math.max(0.18, Math.cos(latitudeRad));
    return [
        (eastValue / speed) * width * 0.014 / cosLat,
        -(northValue / speed) * height * 0.026,
    ];
}
export function wrapLongitudeRad(value) {
    const twoPi = Math.PI * 2;
    return ((value + Math.PI) % twoPi + twoPi) % twoPi - Math.PI;
}
export function clampEquirectangularCenterLatitude(value, zoom) {
    const safeZoom = Math.max(1, zoom);
    const halfSpan = Math.PI / (2 * safeZoom);
    return Math.max(-Math.PI / 2 + halfSpan, Math.min(Math.PI / 2 - halfSpan, value));
}
export function equirectangularScreenToWorldDirection(canvasX, canvasY, width, height, camera) {
    const safeZoom = Math.max(1, camera.zoom);
    const longitude = wrapLongitudeRad(camera.centerLongitudeRad + (canvasX / Math.max(1, width) - 0.5) * Math.PI * 2 / safeZoom);
    const latitude = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, camera.centerLatitudeRad + (0.5 - canvasY / Math.max(1, height)) * Math.PI / safeZoom));
    const cosLatitude = Math.cos(latitude);
    return [
        cosLatitude * Math.cos(longitude),
        cosLatitude * Math.sin(longitude),
        Math.sin(latitude),
    ];
}
export function equirectangularCameraForWorldDirectionAtScreen(direction, canvasX, canvasY, width, height, zoom) {
    const safeZoom = Math.max(1, zoom);
    const longitude = Math.atan2(direction[1], direction[0]);
    const latitude = Math.asin(Math.max(-1, Math.min(1, direction[2])));
    const centerLongitudeRad = wrapLongitudeRad(longitude - (canvasX / Math.max(1, width) - 0.5) * Math.PI * 2 / safeZoom);
    const centerLatitudeRad = clampEquirectangularCenterLatitude(latitude - (0.5 - canvasY / Math.max(1, height)) * Math.PI / safeZoom, safeZoom);
    return { centerLongitudeRad, centerLatitudeRad, zoom: safeZoom };
}
