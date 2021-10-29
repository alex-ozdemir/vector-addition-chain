data.csv: sim.sage
	sage $< | tee $@

incomp_prob.pdf: plot.R data.csv
	Rscript $<
