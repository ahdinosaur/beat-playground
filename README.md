```
sudo apt install portaudio19-dev
```

## resources

- https://github.com/aubio/aubio/blob/3d3d74d4056e3d891aa7c6a1f77fa114a79ccf68/src/tempo/beattracking.h
- http://www.eecs.qmul.ac.uk/~markp/2004/DaviesPlumbley04-ismir.pdf
- http://c4dm.eecs.qmul.ac.uk/papers/2007/Davies07-phdthesis.pdf
- https://s3.amazonaws.com/academia.edu.documents/42988151/Jose_R_Zapata_MultiFeature_BT_preprint.pdf?AWSAccessKeyId=AKIAIWOWYYGZ2Y53UL3A&Expires=1528971098&Signature=YGzuo9LXechzDvAElqkJaFB9Y%2Bc%3D&response-content-disposition=inline%3B%20filename%3DMulti-Feature_Beat_Tracking.pdf
- http://essentia.upf.edu/documentation/reference/std_BeatTrackerMultiFeature.html
- https://dsp.stackexchange.com/questions/12830/easiest-beat-tracking-algorithim
- http://essentia.upf.edu/documentation/reference/std_BeatTrackerDegara.html
- https://www.eecs.qmul.ac.uk/~markp/2011/DaviesDegaraPlumbley11-spl_accepted_postprint.pdf
- \* http://www.eecs.qmul.ac.uk/%7Emarkp/2012/DegaraArgonesRuaPenaTDP12-taslp_accepted.pdf
  - the Dagara
- \* http://www.eecs.qmul.ac.uk/~markp/2009/StarkDaviesPlumbley09-dafx.pdf
  - referenced by the Dagara as the beat algorithm
- \* http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.331.6942
  - referenced by above as the onset detection algorithm


## plan

- window(window_size = 2048, hop_size = 512)
  - iterator to return signals
- energy of a frame
- dft of a frame
- find onsets
- feed into dagara
  - phase deviation
  - spectral difference
  - complex-domain
