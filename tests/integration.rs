use phyto_fsm::generate_fsm;
generate_fsm!("../src/test_data/simple_fsm.puml");

#[cfg(test)]
mod test {

    #[test]
    fn simple_fsm() {
        // TODO where to put the shared test
    }
}
