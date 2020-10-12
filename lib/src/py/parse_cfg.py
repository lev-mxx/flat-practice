from pyformlang.cfg import CFG, Terminal


def parse_cfg(text):
    cfg = CFG.from_text(text)
    produces_epsilon = cfg.generate_epsilon()
    cfg = cfg.to_normal_form()
    productions = list(map(lambda p: (p.head.value, list(map(lambda x: x.value, p.body))), cfg.productions))
    return str(cfg.start_symbol), productions, produces_epsilon
