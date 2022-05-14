import argparse
import json
import matplotlib.cm as cm
import matplotlib.pyplot as plt
import networkx as nx
import numpy as np


# colormap
CMAP = lambda x: cm.Reds(np.clip((0.85 - 0.1) * (x - 0.5) / (1.0 - 0.5), 0.1, 0.85))


def read_dependency(file_path):
    with open(file_path) as f:
        dep_summaries = [json.loads(line) for line in f]
    return dep_summaries


def get_all_subspans(mchain):
    return sorted(list(set(
        list(mchain.keys()) 
        + [name_j for name_i, edges in mchain.items() for name_j in edges]
    )))


def find_steady_states(mchain):
    # construct transition matrix
    all_subspans = get_all_subspans(mchain)
    name2int = {name: idx for idx, name in enumerate(all_subspans)} 
    mat = np.array([
        [
            mchain.get(name_1, dict()).get(name_2, 0.0)
            for name_2 in all_subspans
        ]
        for name_1 in all_subspans
    ])
    
    # fill in sinks
    for idx in range(len(all_subspans)):
        if mat[idx].sum() == 0.0:
            mat[idx, idx] = 1.0
            
    # get steady state matrix by matrix exponentiation
    mat = np.linalg.matrix_power(mat, 4096)
    
    # extract steady state dict
    steady_states = dict()
    for name_1 in all_subspans:
        for name_2 in all_subspans:
            state_prob = mat[name2int[name_1]][name2int[name_2]]
            if state_prob > 0.0:
                d = steady_states.get(name_1, dict())
                d[name_2] = state_prob
                steady_states[name_1] = d
    return steady_states


def dirichlet(count_s, count_f, total_s, total_f, steady_s, steady_f, mu):
    count_all = count_s + count_f
    prob_F = (count_f + mu * steady_f) / (count_all + mu)
    prob_F_NOT = (total_f) / (total_s + total_f)
    return prob_F, prob_F_NOT


def coruscant_analyze(mchains, bernoullis, mu, verbose=0):
    all_inf_scores = dict()
    for span, mchain in mchains.items():
        if span not in bernoullis or '__TOTAL__' not in bernoullis[span]:
            # skip unseen failure
            continue

        if verbose >= 2:
            print(f"")
            print(f"{span}")

        # calculate steady states (MC(i -> S) and MC(i -> F) when S and F are sinks)
        steady_states = find_steady_states(mchain)

        # compute the influential posterior per each subspan 
        all_subspans = get_all_subspans(mchain)
        all_inf_scores[span] = dict()
        for subspan in all_subspans:
            # retrieve bernoully
            bernoulli = bernoullis[span]
            total_s = bernoulli['__TOTAL__'][1] - bernoulli['__TOTAL__'][0]
            total_f = bernoulli['__TOTAL__'][0]

            # recover counts conditioned on failing span
            count_s, count_f = 0, 0
            for failing_subspans, (failing_then_F, total) in bernoulli.items():
                failing_subspans = failing_subspans.split(", ")
                if subspan in failing_subspans:
                    count_s += total - failing_then_F
                    count_f += failing_then_F

            # get relevant MC(i -> S) and MC(i -> F)
            steady_s = steady_states[subspan].get('__SUCCESS_STATE__', 0.0)
            steady_f = steady_states[subspan].get('__FAILURE_STATE__', 0.0)

            # estimate parameter by dirichlet-smoothed MLE
            pi_f, pi_f_not = dirichlet(count_s, count_f, total_s, total_f, steady_s, steady_f, mu)

            # compute likelihood conditioned on influential or non-influential
            score_f = (pi_f ** count_f) * ((1 - pi_f) ** count_s)
            score_f_NOT = (pi_f_not ** count_f) * ((1 - pi_f_not) ** count_s)

            # compute influential posterior
            influence_score = score_f / (score_f + score_f_NOT)
            all_inf_scores[span][subspan] = influence_score

            if verbose >= 2:
                print(f"\t{subspan:40s}: {influence_score:.2e}")
                print(f"\t{'':45s} count(s/f)= ({count_s:5d}, {count_f:5d}), steady_f= {steady_f:.1e}")
                print(f"\t{'':45s} total(s/f)= ({total_s:5d}, {total_f:5d})")
                print(f"\t{'':45s} {pi_f:.2e} -> {score_f:.2e}, {pi_f_not:.2e} -> {score_f_NOT:.2e}")
                print(f"")
    return all_inf_scores


def nonzero(name, bernoullis, all_subspans):
    if name in bernoullis:
        if bernoullis[name]["__TOTAL__"][0] > 0.0:
            return True
    return any(bernoullis[n]["__TOTAL__"][0] > 0.0 for n in all_subspans if n in bernoullis)


def augment_name(name, bernoullis, label=None):
    if label:
        return f"{name}\n({label:.2f})"
    return f"{name}"


def draw_mc_dict(mchain, bernoullis, label_fn=lambda _: None):
    all_subspans = get_all_subspans(mchain)
    for subspan in all_subspans:
        if nonzero(subspan, bernoullis, []):
            prob = bernoullis[subspan]["__TOTAL__"][0] / bernoullis[subspan]["__TOTAL__"][1]
            print(f"\t{subspan}: {prob:.2e}")

    # reconstruct grap
    G = nx.DiGraph()
    edge_labels = {}
    color_map = []
    for name in all_subspans:
        G.add_node(name)
    for name_i, edges in mchain.items():
        for name_j, prob in edges.items():
            G.add_edge(name_i, name_j, weight=prob, label=f"{prob:.1e}")
            edge_labels[(name_i, name_j)] = float(f"{prob:.1e}")
    for node in G:
        label = label_fn(node)
        label = 0.0 if label is None else label
        color_map.append(CMAP(label))
    labels = {
        node: augment_name(node, bernoullis, label=label_fn(node))
        for node in G.nodes()
    }
    
    # plot out
    fig, ax = plt.subplots(figsize=(8, 4))
    pos = nx.spring_layout(G)
    nx.draw(
        G, pos, edge_color='black', width=1, linewidths=1,
        node_size=1000, node_color=color_map, alpha=0.9,
        labels=labels,
        ax=ax
    )
    nx.draw_networkx_edge_labels(
        G, pos,
        edge_labels=edge_labels,
        font_color='blue',
        ax=ax
    )
    plt.show()


def visualize_specific(mchains, bernoullis, span, do_plot, all_inf_scores):
    mchain = mchains[span]
    print(f"================================")
    print(f"span= {span}")

    print(f"")
    print(json.dumps(all_inf_scores[span], sort_keys=True, indent=4))
    
    label_fn = lambda subspan: all_inf_scores[span][subspan]
    err_prob = bernoullis[span] if span in bernoullis else None
    print(f"")
    print(f"failure events: {err_prob}")

    if do_plot:
        draw_mc_dict(mchain, bernoullis, label_fn=label_fn)


if __name__ == '__main__':
    # arguments
    parser = argparse.ArgumentParser(
        description='Analyze dependency summary from coruscant subscriber'
    )
    parser.add_argument('path', type=str, help='path to summary jsons')
    parser.add_argument('--verbose', '-v', action='count', default=0)
    parser.add_argument('--mc', type=str, help='print score on markov chain of a span')
    parser.add_argument('--plot-mc', action="store_true", help='plot score on markov chain of a span')
    parser.add_argument('--print-fault', action="store_true", help='print out fault probabilities')
    parser.add_argument('--spans', action="append", nargs="+", type=str, default=[],
                        help='spans to focus on')
    parser.add_argument('--mu', default=1.0, type=float, help='Dirichlet hyperparameter')
    args = parser.parse_args()

    if len(args.spans) > 0:
        args.spans = [li for l in args.spans for li in l]

    # read summary file and extract
    deps = read_dependency(args.path)
    mchains = deps[-1]['span_markov']
    bernoullis = deps[-1]['fail_bernoulli']

    # analyze
    all_inf_scores = coruscant_analyze(mchains, bernoullis, mu=args.mu, verbose=args.verbose)
    if args.verbose >= 1:
        print(json.dumps(all_inf_scores, sort_keys=True, indent=4))

    # plot markov chain
    if args.mc:
        visualize_specific(mchains, bernoullis, args.mc, args.plot_mc, all_inf_scores)

    # print bernoulli
    if args.print_fault:
        print(f"================================")
        print(f"Non-zero Fault Bernoulli")
        span_list = bernoullis.keys() if len(args.spans) == 0 else args.spans
        for span in span_list:
            if span in bernoullis:
                prob_err = bernoullis[span]['__TOTAL__'][0] / bernoullis[span]['__TOTAL__'][1]
            else:
                prob_err = 0.0
            if len(args.spans) == 0 and prob_err <= 0.0: continue
            print(f"\t{span}: {prob_err}")
