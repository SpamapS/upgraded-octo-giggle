To run benchmarks, in this directory run::

  make
  docker build -t ngtest .
  docker run --rm -ti -p 80:8000 ngtest

This will start a pretend API server up for the tests and make a manifest. Then to benchmark::

  time python3 ../octo.py < api/manifest.txt > /dev/null
  time ../target/release/upgraded-octo-giggle < api/manifest.txt > /dev/null
  time ../zuul-preview/zuul-preview < api/manifest.txt > /dev/null
