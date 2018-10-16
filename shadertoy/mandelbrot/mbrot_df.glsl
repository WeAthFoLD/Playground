// WeAthFolD, 2018.10.16
// reference: 
//  - http://www.iquilezles.org/www/articles/distancefractals/distancefractals.htm
//  - https://www.shadertoy.com/view/lsX3W4
const int MAX_ITER = 300;

float mandelbrot(float re_c, float im_c) {
    vec2 z = vec2(0.0, 0.0);
    vec2 dz = vec2(0.0, 0.0);

    float m2 = 0.0;
    float di = 1.0;
    for (int i = 0; i < MAX_ITER; ++i) {
        dz = 2.0 * vec2(z.x * dz.x - z.y * dz.y, z.x * dz.y + z.y * dz.x) + vec2(1.0, 0.0);
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + vec2(re_c, im_c);
        m2 = dot(z, z);
        if (m2 > 1e8) {
            di = 0.0;
            break;
        }
    }
    if (di > 0.5)
        return 0.0;
    float d = 0.5 * sqrt(dot(z, z) / dot(dz, dz)) * log(dot(z, z));
    return d;
}

void main() {
    vec2 uv = (gl_FragCoord.xy / iResolution.x - 0.5);
    float time = iGlobalTime;
    // float scaleFactor = 8.0;
    float scaleFactor = 1.0 + pow(2.0, mod(time * 2.0, 20.0));
    float x_offset = -0.7492;
    
    float re_c = uv.x * 4.0 / scaleFactor + x_offset;
    float im_c = uv.y * 4.0 / scaleFactor + 0.1;
    float dist = mandelbrot(re_c, im_c);
    float soft = clamp(pow(8.0 * dist * scaleFactor, 0.2), 0.0, 1.0);

    gl_FragColor = vec4(vec3(soft), 1);
}