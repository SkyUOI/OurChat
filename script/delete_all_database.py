import basic


command = """
psql -d postgres < <( psql -Atc "select 'drop database \"'||datname||'\";' from pg_database where datistemplate=false AND datname <> 'postgres';")
"""
basic.msg_system(command)
