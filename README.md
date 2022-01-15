# Generation of word digrams 2 grams pt_PT in Rust
pt_PT digrams n_grams and word frequency files. 

## Description
This small project is a efficient way of generating a digram file and frequency of words file for pt_PT (Portuguese) from the European Parliament Proceedings Parallel Corpus 1996-2011 (see reference below for details). <br>
<br>

**The processing that I have made:** <br>
1. The text file was divided into phrases and word space delimited.

2. Each word was checked for valid chars in Portuguese with a reg_ex.

3. Each word was checked if it was a valid Portuguese word, with the HunSpell dictionary pt_PT from 2021.12.25 (see Project Natura, below).

4. Because the text is pre orthographic treaty, I tried to save the words that had one more muted 'c' or 'p' char and words that started with an uppercase like "alemanha" vs "Alemanha" or opec vs OPEC.

5. Then I have made some mapping between wrong written words in terms of just one accent sign.

This was possible because HunSpell has a suggest() function in it's API that returns close lexical valid words. The previous similarity process seemed to me a "safe" and simple process to do. <br>
<br>

**Note:** <br>
The suggest() function in HunSpell is rather slow because it has to try every single permutation for a predetermined distance. SymSpell algorithm is faster in this regard. So I implemented a cache over the correct and incorrect words, to lower the number of calls made to the suggest() function from HunSpell. It worked the processing time went down from 3 H or 4 H to 20 minutes, on a single core.


## References 
1. Projeto Natura - Hunspell - dictionary pt_PT in Portuguese <br>
   [https://natura.di.uminho.pt/wiki/doku.php?id=dicionarios:main](https://natura.di.uminho.pt/wiki/doku.php?id=dicionarios:main)

2. European Parliament Proceedings Parallel Corpus 1996-2011 - Portuguese <br>
   [https://www.statmt.org/europarl/](https://www.statmt.org/europarl/)

3. SymSpell crate in Rust <br>
   [https://github.com/reneklacan/symspell](https://github.com/reneklacan/symspell)

4. Original SymSpell by Wolfgarbe in C# <br>
   It doesn't have a pt_PT word frequency dictionary and digrams. <br>
   [https://github.com/wolfgarbe/SymSpell](https://github.com/wolfgarbe/SymSpell)


## License
I place my code under MIT Open Source License. <br>
But the license of the digrams and the words frequency of the text are in there respective licenses. In reference 2. is said about the big 320 MB pt_PT text "We are not aware of any copyright restrictions of the material.". See the reference European Parliament Proceedings Parallel Corpus 1996-2011 above.


## Have fun!
Best regards, <br>
João Nuno Carvalho

