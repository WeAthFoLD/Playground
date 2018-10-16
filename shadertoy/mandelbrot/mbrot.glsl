// WeAthFolD, 2018.10.16
// reference: http://jonisalonen.com/2013/lets-draw-the-mandelbrot-set/
const int MAX_ITER = 100;

void main() {
    vec2 uv = (gl_FragCoord.xy / iResolution.x - 0.5);
    float time = iGlobalTime;
    float scaleFactor = 1.0 + (sin(time * 0.2) + 1.0) * 18000.0;
    float x_offset = -0.7492;
    
    float re_c = uv.x * 4.0 / scaleFactor + x_offset;
    float im_c = uv.y * 4.0 / scaleFactor + 0.1;

    float re = 0.0;
    float im = 0.0;

    int iter = 0;
    while (re * re + im * im <= 4.0 && iter < MAX_ITER) {
        float re0 = re * re - im * im + re_c;
        im = 2.0 * re * im + im_c;
        re = re0;
        ++iter;
    }

    vec4 res = iter < MAX_ITER ? vec4(1,1,1,1) : vec4(0,0,0,1);

    gl_FragColor = res;
}