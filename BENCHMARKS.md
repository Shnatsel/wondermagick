# wondermagick benchmarks

We compare `wondermagick` against `imagemagick` in their default configurations. This includes a ~5% performance penalty from the hardened memory allocator that `wondermagick` defaults to.

Note that you could improve the performance of both of them using arcane compilation flags. We're not going to do that here for either project. The point of this comparison is to show what kind of performance you might expect by simply installing a distribution package.

### Resizing JPEG

Input file: <https://commons.wikimedia.org/wiki/File:Sun_over_Lake_Hawea,_New_Zealand.jpg>

```
$ ARGS='Sun_over_Lake_Hawea,_New_Zealand.jpg -resize 25% out.jpg' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert Sun_over_Lake_Hawea,_New_Zealand.jpg -resize 25% out.jpg
  Time (mean ± σ):     216.2 ms ±   1.7 ms    [User: 526.2 ms, System: 116.9 ms]
  Range (min … max):   212.9 ms … 218.2 ms    13 runs
 
Benchmark 2: wm-convert Sun_over_Lake_Hawea,_New_Zealand.jpg -resize 25% out.jpg
  Time (mean ± σ):     142.1 ms ±   1.7 ms    [User: 204.9 ms, System: 40.2 ms]
  Range (min … max):   139.2 ms … 145.0 ms    21 runs
 
Summary
  wm-convert Sun_over_Lake_Hawea,_New_Zealand.jpg -resize 25% out.jpg ran
    1.52 ± 0.02 times faster than convert Sun_over_Lake_Hawea,_New_Zealand.jpg -resize 25% out.jpg
```

### Resizing PNG

Input file: <https://commons.wikimedia.org/wiki/File:Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png>

```
$ ARGS='"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -resize 25% out.png' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -resize 25% out.png
  Time (mean ± σ):     124.4 ms ±   3.5 ms    [User: 298.9 ms, System: 50.7 ms]
  Range (min … max):   121.4 ms … 134.4 ms    22 runs
 
Benchmark 2: wm-convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -resize 25% out.png
  Time (mean ± σ):      48.9 ms ±   0.9 ms    [User: 67.6 ms, System: 18.5 ms]
  Range (min … max):    46.9 ms …  51.7 ms    60 runs
 
Summary
  wm-convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -resize 25% out.png ran
    2.55 ± 0.09 times faster than convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -resize 25% out.png
```

### Resizing Lossy WebP

Input file: <https://commons.wikimedia.org/wiki/File:Museum_in_Chennai.webp>

```
ARGS='Museum_in_Chennai.webp -resize 50% out.webp' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert Museum_in_Chennai.webp -resize 50% out.webp
  Time (mean ± σ):     281.5 ms ±   5.5 ms    [User: 556.4 ms, System: 220.3 ms]
  Range (min … max):   272.8 ms … 290.6 ms    10 runs
 
Benchmark 2: wm-convert Museum_in_Chennai.webp -resize 50% out.webp
  Time (mean ± σ):     312.7 ms ±   3.1 ms    [User: 342.9 ms, System: 47.4 ms]
  Range (min … max):   307.5 ms … 319.2 ms    10 runs
 
Summary
  convert Museum_in_Chennai.webp -resize 50% out.webp ran
    1.11 ± 0.02 times faster than wm-convert Museum_in_Chennai.webp -resize 50% out.webp
```

### Resizing Lossless WebP

Input file: <https://commons.wikimedia.org/wiki/File:Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png> converted to WebP with `convert in.png -quality 100 exoplanet-lossless.webp`

```
ARGS='exoplanet-lossless.webp -resize 50% -quality 100 out.webp' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert exoplanet-lossless.webp -resize 50% -quality 100 out.webp
  Time (mean ± σ):     374.9 ms ±   2.0 ms    [User: 531.5 ms, System: 112.8 ms]
  Range (min … max):   372.4 ms … 379.1 ms    10 runs
 
Benchmark 2: wm-convert exoplanet-lossless.webp -resize 50% -quality 100 out.webp
  Time (mean ± σ):     357.5 ms ±   2.9 ms    [User: 361.8 ms, System: 45.8 ms]
  Range (min … max):   354.7 ms … 362.7 ms    10 runs
 
Summary
  wm-convert exoplanet-lossless.webp -resize 50% -quality 100 out.webp ran
    1.05 ± 0.01 times faster than convert exoplanet-lossless.webp -resize 50% -quality 100 out.webp
```

### Converting PNG to Lossless WebP for better compression

Input file: <https://commons.wikimedia.org/wiki/File:Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png>

```
ARGS='"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -quality 100 out.webp' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -quality 100 out.webp
  Time (mean ± σ):      1.215 s ±  0.007 s    [User: 1.147 s, System: 0.069 s]
  Range (min … max):    1.206 s …  1.224 s    10 runs
 
Benchmark 2: wm-convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -quality 100 out.webp
  Time (mean ± σ):      1.135 s ±  0.003 s    [User: 1.080 s, System: 0.054 s]
  Range (min … max):    1.131 s …  1.141 s    10 runs
 
Summary
  wm-convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -quality 100 out.webp ran
    1.07 ± 0.01 times faster than convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -quality 100 out.webp
```

### Converting Lossless WebP to PNG for wider compatibility

Input file: result from the previous step

```
ARGS='exoplanet-lossless.webp out.png' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert exoplanet-lossless.webp out.png
  Time (mean ± σ):     277.4 ms ±   3.0 ms    [User: 320.7 ms, System: 46.1 ms]
  Range (min … max):   275.0 ms … 283.8 ms    10 runs
 
Benchmark 2: wm-convert exoplanet-lossless.webp out.png
  Time (mean ± σ):     173.2 ms ±   2.2 ms    [User: 150.8 ms, System: 22.2 ms]
  Range (min … max):   171.2 ms … 178.6 ms    16 runs
 
Summary
  wm-convert exoplanet-lossless.webp out.png ran
    1.60 ± 0.03 times faster than convert exoplanet-lossless.webp out.png
```

### Converting a lossy WebP to PNG with light compression so that a legacy system could read it

Input file: <https://commons.wikimedia.org/wiki/File:Museum_in_Chennai.webp>

```
$ ARGS='Museum_in_Chennai.webp -quality 1 out.png' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert Museum_in_Chennai.webp -quality 1 out.png
  Time (mean ± σ):     379.9 ms ±   6.7 ms    [User: 420.0 ms, System: 77.0 ms]
  Range (min … max):   368.5 ms … 386.3 ms    10 runs
 
Benchmark 2: wm-convert Museum_in_Chennai.webp -quality 1 out.png
  Time (mean ± σ):     236.9 ms ±   6.3 ms    [User: 149.5 ms, System: 85.6 ms]
  Range (min … max):   229.3 ms … 251.9 ms    12 runs
 
Summary
  wm-convert Museum_in_Chennai.webp -quality 1 out.png ran
    1.60 ± 0.05 times faster than convert Museum_in_Chennai.webp -quality 1 out.png
```

### Converting PNG to JPEG

Input file: <https://commons.wikimedia.org/wiki/File:%22Wind_Mountain%22_Columbia_R_-_NARA_-_102278851_(page_1).png>

```
ARGS='"_Wind_Mountain__Columbia_R_-_NARA_-_102278851_(page_1).png" out.jpg' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert "_Wind_Mountain__Columbia_R_-_NARA_-_102278851_(page_1).png" out.jpg
  Time (mean ± σ):     549.9 ms ±   1.7 ms    [User: 454.0 ms, System: 95.8 ms]
  Range (min … max):   548.2 ms … 552.7 ms    10 runs
 
Benchmark 2: wm-convert "_Wind_Mountain__Columbia_R_-_NARA_-_102278851_(page_1).png" out.jpg
  Time (mean ± σ):     331.6 ms ±   2.0 ms    [User: 301.2 ms, System: 30.3 ms]
  Range (min … max):   329.2 ms … 336.1 ms    10 runs
 
Summary
  wm-convert "_Wind_Mountain__Columbia_R_-_NARA_-_102278851_(page_1).png" out.jpg ran
    1.66 ± 0.01 times faster than convert "_Wind_Mountain__Columbia_R_-_NARA_-_102278851_(page_1).png" out.jpg
```

### Converting PNG to GIF

Input file: <https://commons.wikimedia.org/wiki/File:Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png>

We use [NeuQuant](https://scientificgems.wordpress.com/stuff/neuquant-fast-high-quality-image-quantization/) algorithm to produce better-looking results than what imagemagick is capable of.

```
$ ARGS='"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" out.gif' hyperfine --warmup=3 "convert $ARGS" "wm-convert $ARGS"
Benchmark 1: convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" out.gif
  Time (mean ± σ):     642.7 ms ±   8.4 ms    [User: 700.5 ms, System: 97.4 ms]
  Range (min … max):   631.9 ms … 659.5 ms    10 runs
 
Benchmark 2: wm-convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" out.gif
  Time (mean ± σ):     411.4 ms ±   2.3 ms    [User: 387.6 ms, System: 23.8 ms]
  Range (min … max):   408.0 ms … 416.1 ms    10 runs
 
Summary
  wm-convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" out.gif ran
    1.56 ± 0.02 times faster than convert "Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" out.gif
```

All measurements were taken on commit `373e1c69196741523b11015c5ecb6f4b44ffe32a` on Rust 1.91.1 an AMD Zen 4 desktop CPU. Imagemagick version `8:6.9.11.60+dfsg-1.3ubuntu0.22.04.5` from the Ubuntu repositories was used as a point of reference. Imagemagick version `7.1.1.43+dfsg1-1` from Ubuntu 25.04 repositories was also measured, but the results are nearly identical to the older version, so they are omitted for brevity.
