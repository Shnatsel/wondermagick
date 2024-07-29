# wondermagick benchmarks

We compare `wondermagick` against `imagemagick` in their default configurations. 

Note that you could improve the performance of both of them using arcane compilation flags. We're not going to do that here. The point of this comparison is to show what kind of performance a non-expert user might expect.

Due to `wondermagick` being still in development, the numbers should be taken with a grain of salt. Think "extremely promising in early tests", *not* "already faster across the board" - even though we believe it has the potential to be!

### Thumbnailing a JPEG:

```
$ hyperfine --warmup=3 './wm-convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg' 'convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg'
Benchmark 1: ./wm-convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg
  Time (mean ± σ):      99.2 ms ±   3.9 ms    [User: 71.9 ms, System: 27.1 ms]
  Range (min … max):    97.1 ms … 108.9 ms    27 runs
 
Benchmark 2: convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg
  Time (mean ± σ):     160.7 ms ±   1.0 ms    [User: 114.0 ms, System: 56.4 ms]
  Range (min … max):   159.8 ms … 164.4 ms    18 runs
 
Summary
  ./wm-convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg ran
    1.62 ± 0.06 times faster than convert ~/Sun_over_Lake_Hawea,_New_Zealand.jpg -thumbnail 120x120 ~/out.jpg
```

### Thumbnailing a PNG:

```
hyperfine --warmup=3 './wm-convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png' 'convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png'
Benchmark 1: ./wm-convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png
  Time (mean ± σ):      29.3 ms ±   1.3 ms    [User: 19.8 ms, System: 9.4 ms]
  Range (min … max):    28.1 ms …  31.9 ms    92 runs
 
Benchmark 2: convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png
  Time (mean ± σ):      89.1 ms ±   0.3 ms    [User: 81.7 ms, System: 27.2 ms]
  Range (min … max):    88.8 ms …  89.8 ms    33 runs
 
Summary
  ./wm-convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png ran
    3.04 ± 0.13 times faster than convert ~/"Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png" -thumbnail 120x120 ~/out.png
```

### Converting a WebP to PNG with light compression so that a legacy system could read it:

Curiously, we come out ahead on this test despite our WebP decoder being slow. We more than make up for it with the extremely fast encoding of lightly compressed PNGs, and *still* produce a 4x smaller file than `imagemagick`.

```
hyperfine --warmup=3 './wm-convert ~/Museum_in_Chennai.webp ~/out.png' 'convert -quality 1 ~/Museum_in_Chennai.webp ~/out.png'
Benchmark 1: ./wm-convert ~/Museum_in_Chennai.webp ~/out.png
  Time (mean ± σ):     265.5 ms ±   0.9 ms    [User: 231.9 ms, System: 33.1 ms]
  Range (min … max):   264.8 ms … 267.1 ms    11 runs
 
Benchmark 2: convert -quality 1 ~/Museum_in_Chennai.webp ~/out.png
  Time (mean ± σ):     381.2 ms ±   6.0 ms    [User: 335.8 ms, System: 75.3 ms]
  Range (min … max):   375.4 ms … 388.5 ms    10 runs
 
Summary
  ./wm-convert ~/Museum_in_Chennai.webp ~/out.png ran
    1.44 ± 0.02 times faster than convert -quality 1 ~/Museum_in_Chennai.webp ~/out.png
```

We do not (yet) test encoding speed for other formats and scenarios because the encoding parameters are not yet aligned between the two implementations. It is easy to show improvements on benchmarks by doing less work and producing a larger file - we do not want to do that! That's also why we don't publish resize or conversion tests yet.

The gist of encoding performance is that encoding PNG, GIF, WebP and AVIF is fast. Encoding JPEG is slow, but it is easy to speed up.

All measurements were taken on commit `d6bfe2956281f9cef7c4e332599adb0ffa89d8bd` on an AMD Zen 4 desktop CPU.

Test files used:

- <https://commons.wikimedia.org/wiki/File:Sun_over_Lake_Hawea,_New_Zealand.jpg>
- <https://commons.wikimedia.org/wiki/File:Exoplanet_Phase_Curve_(Diagram)_(01HK57P2YHV18MMV0RG5N7HY70).png>
- <https://commons.wikimedia.org/wiki/File:Museum_in_Chennai.webp>
