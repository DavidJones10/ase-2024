# Compute Time Comparisons Between Time-Domain and Frequency-Domain Convolution

## Compares the 2 Domains with varying impulse response (IR) sizes with a constant block size of 1024:
![Image Title](https://raw.githubusercontent.com/YourUsername/YourRepositoryName/main/Assets/IR-Length-Plots.png)
#### As the length of the IR increases, the compute time of the time-domain convolution increases exponentially, while the compute time of the frequency domain only increases logarithmically

## Compares the 2 Domains with varying block sizes with a constant IR length of 1000:
![Image Title](https://raw.githubusercontent.com/YourUsername/YourRepositoryName/main/Assets/Block-Size-Plots.png)
#### As the block size increases, the compute time in both domains improves. The frequency-domain convolver, however, improves to a much greater extent.


