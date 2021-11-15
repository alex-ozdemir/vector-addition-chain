library(tidyverse)
library(scales)
d <- read_csv("data.csv")
d$n <- as.factor(d$n)
ggplot(d, mapping = aes(x = bits, y = p, color = n, shape = n)) +
  geom_point() +
  geom_smooth() +
  scale_y_continuous(trans = log2_trans(),
                     breaks = trans_breaks("log2", function(x) 2^x),
                     labels = trans_format("log2", math_format(2^.x))) +
  labs(
    y = "Incompletenss error",
    x = "Scalar bits",
    shape = "Vector length",
    color = "Vector length"
  )
ggsave("incomp_prob.pdf", width = 6, height=4, units ="in")
