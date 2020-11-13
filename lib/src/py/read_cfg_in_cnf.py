from pyformlang.cfg import CFG, Terminal


def parse_cfg(text):
    cfg = CFG.from_text(text)
    produces_epsilon = cfg.generate_epsilon()
    nonterminals = list(map(lambda v: v.value, cfg.variables))
    cfg = cfg.to_normal_form()
    productions = list(map(lambda p: (p.head.value, list(map(lambda x: x.value, p.body))), cfg.productions))
    return cfg.start_symbol.value, nonterminals, productions, produces_epsilon
