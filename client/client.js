const algoliasearch = require('algoliasearch');

const stHost = {
  protocol: 'http',
  url: 'localhost:3000',
  accept: 1
};

const stHost2 = {
  protocol: 'http',
  url: 'localhost:3000',
  accept: 2
};

const client = algoliasearch("applicationId", "apiKey");

client.transporter.hosts = [stHost, stHost2];

//console.log(client.transporter.hosts);

const index = client.initIndex('your_index_name');

const objects = [
  {
    objectID: 1,
    name: 'Foo',
    title: 'El foo de la fuera',
    summary: 'El fuero de la fuera fueron fuerar con pontito...'
  },
];

index.saveObjects(objects).then(({ objectIDs }) => {
  console.log(objectIDs);
}).catch(err => {
  console.log(JSON.stringify(err, null, 2));
});

index.search('Fo').then(({ hits }) => {
  console.log(hits);
}).catch(err => {
  //  console.log(err);
  console.log(JSON.stringify(err, null, 2));

});
