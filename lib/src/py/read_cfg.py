from pyformlang.cfg import CFG


def read_cfg(text):
    cfg = CFG.from_text(text)
    productions = list(map(lambda p: (p.head.value, list(map(lambda x: x.value, p.body))), cfg.productions))
    return cfg.start_symbol.value, productions
