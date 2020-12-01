from pyformlang.regular_expression import Regex

def regex_to_edges(regex):
    regex = Regex(regex)
    dfa = regex.to_epsilon_nfa().to_deterministic().minimize()
    states = dict()
    for i, state in enumerate(dfa.states):
        states[state] = i

    edges = []
    for frm, label, to in dfa._transition_function.get_edges():
        edges.append((states[frm], states[to], str(label)))
    finals = []
    for state in dfa._final_states:
        finals.append(states[state])
    return states[dfa.start_state], finals, edges
