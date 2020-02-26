const execa = require('execa');

module.exports = class EpirustService{
    start(numberOfAgents) {
        (async () => {
            try {
                const {stdout} = await execa('./external/epirust', [numberOfAgents]);
                console.log(stdout);
            } catch (error) {
                throw new Error("Failed spawning epirust engine - " + error);
            }
        })();
    }
};