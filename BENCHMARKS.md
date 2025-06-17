# wondermagick benchmarks

We compare `wondermagick` against `imagemagick` in their default configurations. 

Note that you could improve the performance of both of them using arcane compilation flags. We're not going to do that here. The point of this comparison is to show what kind of performance a non-expert user might expect.

Due to `wondermagick` being still in development, the numbers should be taken with a grain of salt. Think "extremely promising in early tests", *not* "already faster across the board" - even though we believe it has the potential to be!

### Thumbnailing a JPEG

Input file: <https://commons.wikimedia.org/wiki/File:Sun_over_Lake_Hawea,_New_Zealand.jpg>

```
$ hyperfine --warmup=3 './wm-convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg' 'convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg'
Benchmark 1: ./wm-convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg
  Time (mean ± σ):     102.3 ms ±   3.1 ms    [User: 74.4 ms, System: 27.8 ms]
  Range (min … max):    99.3 ms … 109.7 ms    27 runs
 
Benchmark 2: convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg
  Time (mean ± σ):     168.1 ms ±   2.9 ms    [User: 115.6 ms, System: 61.5 ms]
  Range (min … max):   163.4 ms … 171.5 ms    18 runs
 
Summary
  ./wm-convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg ran
    1.64 ± 0.06 times faster than convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg
```

### Thumbnailing a PNG

Input file: <https://commons.wikimedia.org/wiki/File:Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png>

```
$ hyperfine --warmup=3 './wm-convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png' 'convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png'
Benchmark 1: ./wm-convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png
  Time (mean ± σ):      25.5 ms ±   1.1 ms    [User: 16.5 ms, System: 9.0 ms]
  Range (min … max):    24.1 ms …  27.8 ms    110 runs
 
Benchmark 2: convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png
  Time (mean ± σ):      89.7 ms ±   1.5 ms    [User: 82.4 ms, System: 24.5 ms]
  Range (min … max):    88.0 ms …  92.8 ms    33 runs
 
Summary
  ./wm-convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png ran
    3.51 ± 0.16 times faster than convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png
```

### Converting a WebP to PNG with light compression so that a legacy system could read it

Curiously, we come out ahead on this test despite our lossy WebP decoder being slow. We more than make up for it with the extremely fast encoding of lightly compressed PNGs, and *still* produce a 4x smaller file than `imagemagick`.

Input file: <https://commons.wikimedia.org/wiki/File:Museum_in_Chennai.webp>

```
$ hyperfine --warmup=3 './wm-convert ~/Museum_in_Chennai.webp ~/out.png' 'convert -quality 1 ~/Museum_in_Chennai.webp ~/out.png'
Benchmark 1: ./wm-convert ~/Museum_in_Chennai.webp ~/out.png
  Time (mean ± σ):     246.4 ms ±   4.7 ms    [User: 213.3 ms, System: 30.0 ms]
  Range (min … max):   239.6 ms … 256.5 ms    12 runs
 
Benchmark 2: convert -quality 1 ~/Museum_in_Chennai.webp ~/out.png
  Time (mean ± σ):     386.8 ms ±   6.1 ms    [User: 423.4 ms, System: 75.9 ms]
  Range (min … max):   373.4 ms … 395.0 ms    10 runs
 
Summary
  ./wm-convert ~/Museum_in_Chennai.webp ~/out.png ran
    1.57 ± 0.04 times faster than convert -quality 1 ~/Museum_in_Chennai.webp ~/out.png
```

We do not (yet) test encoding speed for other formats and scenarios because the encoding parameters are not yet aligned between the two implementations. It is easy to show improvements on benchmarks by doing less work and producing a larger file - we do not want to do that! That's also why we don't publish resize or conversion tests yet.

The gist of encoding performance is that encoding PNG, GIF, WebP and AVIF is fast. Encoding JPEG is slow, but it is easy to speed up.

All measurements were taken on commit `28ec08d8c608bb852ee5ec24195c14ad482dd5b7` on Rust 1.86 an AMD Zen 4 desktop CPU. Imagemagick version `8:6.9.11.60+dfsg-1.3ubuntu0.22.04.5` from the Ubuntu repositories was used as a point of reference. Imagemagick version `7.1.1.43+dfsg1-1` from Ubuntu 25.04 repositories was also measured, but the results are nearly identical to the older Ubuntu version, so they are omitted for brevity.
