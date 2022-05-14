for mu in {1.0,1.5,2.25,3.37,5.06,7.59,11.39,17.08,25.62,38.44,57.66,86.49,129.74,194.61,291.92,437.89,656.84,985.26,1477.89,2216.83}; do echo ">>> ${mu}"; python scripts/analyze.py dependency_summary.jsons --plot-mc call_a --mu $mu; done

for n in {1,3,5,7}; do for f in {50,75,87.5,93.75}; do echo ">>> $n, $f"; python scripts/analyze.py ../../openraft/dep/netem_${n}_1_${f}.jsons --print-fault --spans call_core client_write line_rate_loop send_append_entries send_vote_req try_drain_raft_rx; done; done
