library(tidyverse)
d <- read_csv("data.csv")
d$n <- as.factor(d$n)
ggplot(d, mapping = aes(x = bits, y = p, color = n, shape = n)) +
  geom_point() +
  geom_smooth() +
  scale_y_continuous(trans = "log2",labels = function(x) format(x, scientific = TRUE)) +
  labs(
    y = "Incompletenss error",
    x = "Scalar bits",
    shape = "Vector length",
    color = "Vector length"
  )
ggsave("incomp_prob.pdf", width = 6, height=4, units ="in")