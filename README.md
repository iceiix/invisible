# invisible

Test case for OpenGL rendering issue, see:

* https://github.com/iceiix/steven/issues/25 Invisible players/models
* https://github.com/iceiix/steven/pull/33 Isolating invisible models bug

The sun is supposed to render on top of the sky:

![Sun](https://user-images.githubusercontent.com/43691553/49110303-f9a5f880-f241-11e8-97a9-17f2d6c58404.png)

If the problem occurs, then the sun invisible, only rendering the sky background:

![No sun](https://user-images.githubusercontent.com/43691553/48989375-cb5bd800-f0de-11e8-83e8-c46abe222878.png)
