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
